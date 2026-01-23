// File: src/parser.rs
//
// Recursive descent parser for the Ruff programming language.
// Transforms a sequence of tokens into an Abstract Syntax Tree (AST).
//
// The parser implements a traditional recursive descent parsing strategy with
// operator precedence for expressions. It supports:
// - Variable declarations (let, mut, const, shorthand :=)
// - Function definitions
// - Enum definitions
// - Control flow (if/else, match, loop, for, try/except)
// - Expression parsing with proper operator precedence
//
// The parser uses a single-token lookahead and advances through the token stream
// as it builds the AST.

use crate::ast::{Expr, Stmt};
use crate::lexer::{Token, TokenKind};
use std::fs;
use std::path::Path;
use std::time::Instant;
use std::sync::{Arc, Mutex};

/// Parser maintains position in token stream and provides methods to parse statements and expressions
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Creates a new parser from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Peek at the current token without consuming it
    fn peek(&self) -> &TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    /// Consume and return the current token, then advance to the next
    fn advance(&mut self) -> &TokenKind {
        let tok = self.tokens.get(self.pos).map(|t| &t.kind).unwrap_or(&TokenKind::Eof);
        self.pos += 1;
        tok
    }

    /// Parse the entire token stream into a vector of statements
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !matches!(self.peek(), TokenKind::Eof) {
            // Skip semicolons between statements
            if matches!(self.peek(), TokenKind::Punctuation(';')) {
                self.advance();
                continue;
            }
            
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            } else {
                break;
            }
        }
        stmts
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.peek() {
            TokenKind::Keyword(k) if k == "let" || k == "mut" => self.parse_let(),
            TokenKind::Keyword(k) if k == "const" => self.parse_const(),
            TokenKind::Keyword(k) if k == "func" => self.parse_func(),
            TokenKind::Keyword(k) if k == "enum" => self.parse_enum(),
            TokenKind::Keyword(k) if k == "struct" => self.parse_struct(),
            TokenKind::Keyword(k) if k == "import" || k == "from" => self.parse_import(),
            TokenKind::Keyword(k) if k == "export" => self.parse_export(),
            TokenKind::Keyword(k) if k == "return" => {
                self.advance();
                let expr = if !matches!(self.peek(), TokenKind::Punctuation(';')) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                Some(Stmt::Return(expr))
            }
            TokenKind::Keyword(k) if k == "if" => self.parse_if(),
            TokenKind::Keyword(k) if k == "try" => self.parse_try_except(),
            TokenKind::Keyword(k) if k == "match" => self.parse_match(),
            TokenKind::Keyword(k) if k == "loop" => self.parse_loop(),
            TokenKind::Keyword(k) if k == "for" => self.parse_for(),
            TokenKind::Identifier(_) => {
                // Check for variable assignment (name := expr or expr[...] := expr)
                // We need to look ahead and parse an expression to see if it's followed by :=
                let saved_pos = self.pos;
                if let Some(expr) = self.parse_expr() {
                    // Check if next token is :=
                    if matches!(self.peek(), TokenKind::Operator(op) if op == ":=") {
                        self.advance(); // consume :=
                        
                        // Parse := as assignment (create or update)
                        // The interpreter will decide whether to create new or update existing
                        let value = self.parse_expr()?;
                        Some(Stmt::Assign {
                            target: expr,
                            value,
                        })
                    } else {
                        // Not an assignment, restore position and parse as expression statement
                        self.pos = saved_pos;
                        self.parse_expr().map(Stmt::ExprStmt)
                    }
                } else {
                    None
                }
            }
            _ => self.parse_expr().map(Stmt::ExprStmt),
        }
    }

    fn parse_enum(&mut self) -> Option<Stmt> {
        self.advance(); // enum
        let name = match self.advance() {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        self.advance(); // {
        let mut variants = Vec::new();
        while let TokenKind::Identifier(v) = self.peek() {
            variants.push(v.clone());
            self.advance();
            if matches!(self.peek(), TokenKind::Punctuation(',')) {
                self.advance();
            } else {
                break;
            }
        }
        self.advance(); // }
        Some(Stmt::EnumDef { name, variants })
    }

    fn parse_struct(&mut self) -> Option<Stmt> {
        self.advance(); // struct
        let name = match self.advance() {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        
        self.advance(); // {
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        
        while !matches!(self.peek(), TokenKind::Punctuation('}')) && !matches!(self.peek(), TokenKind::Eof) {
            // Check if this is a method definition
            if matches!(self.peek(), TokenKind::Keyword(k) if k == "func") {
                if let Some(method) = self.parse_func() {
                    methods.push(method);
                }
            } else if let TokenKind::Identifier(field_name) = self.peek() {
                // Parse field: name: type
                let field_name = field_name.clone();
                self.advance();
                
                let field_type = self.parse_type_annotation();
                fields.push((field_name, field_type));
                
                // Consume optional comma
                if matches!(self.peek(), TokenKind::Punctuation(',')) {
                    self.advance();
                }
                // If no comma, continue to next field or closing brace
            } else {
                // Unknown token, skip it to avoid infinite loop
                self.advance();
            }
        }
        
        self.advance(); // }
        Some(Stmt::StructDef { name, fields, methods })
    }

    fn parse_let(&mut self) -> Option<Stmt> {
        // Handle 'mut', 'let', or bare identifier with :=
        let is_mut = match self.peek() {
            TokenKind::Keyword(k) if k == "mut" => {
                self.advance();
                true
            }
            TokenKind::Keyword(k) if k == "let" => {
                self.advance();
                false
            }
            _ => false, // Plain identifier (e.g., val := ...)
        };
        
        let name = match self.advance() {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        
        // Parse optional type annotation (: type)
        let type_annotation = self.parse_type_annotation();
        
        self.advance(); // :=
        let value = self.parse_expr()?;
        Some(Stmt::Let {
            name,
            value,
            mutable: is_mut,
            type_annotation,
        })
    }

    fn parse_const(&mut self) -> Option<Stmt> {
        self.advance(); // const
        let name = match self.advance() {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        
        // Parse optional type annotation (: type)
        let type_annotation = self.parse_type_annotation();
        
        self.advance(); // :=
        let value = self.parse_expr()?;
        Some(Stmt::Const { name, value, type_annotation })
    }

    fn parse_func(&mut self) -> Option<Stmt> {
        self.advance(); // func
        let name = match self.advance() {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        self.advance(); // (
        let mut params = Vec::new();
        let mut param_types = Vec::new();
        
        while let TokenKind::Identifier(p) = self.peek() {
            params.push(p.clone());
            self.advance();
            
            // Parse optional type annotation for parameter
            let param_type = self.parse_type_annotation();
            param_types.push(param_type);
            
            if matches!(self.peek(), TokenKind::Punctuation(',')) {
                self.advance();
            } else {
                break;
            }
        }
        self.advance(); // )
        
        // Parse optional return type annotation (-> type)
        let return_type = if matches!(self.peek(), TokenKind::Operator(op) if op == "->") {
            self.advance(); // ->
            match self.peek() {
                TokenKind::Keyword(k) if k == "int" => {
                    self.advance();
                    Some(crate::ast::TypeAnnotation::Int)
                }
                TokenKind::Keyword(k) if k == "float" => {
                    self.advance();
                    Some(crate::ast::TypeAnnotation::Float)
                }
                TokenKind::Keyword(k) if k == "string" => {
                    self.advance();
                    Some(crate::ast::TypeAnnotation::String)
                }
                TokenKind::Keyword(k) if k == "bool" => {
                    self.advance();
                    Some(crate::ast::TypeAnnotation::Bool)
                }
                _ => None,
            }
        } else {
            None
        };
        
        self.advance(); // {
        let mut body = Vec::new();
        while !matches!(self.peek(), TokenKind::Punctuation('}')) {
            if let Some(stmt) = self.parse_stmt() {
                body.push(stmt);
            } else {
                break;
            }
        }
        self.advance(); // }
        Some(Stmt::FuncDef { 
            name, 
            param_types,
            return_type,
            params, 
            body 
        })
    }

    fn parse_match(&mut self) -> Option<Stmt> {
        self.advance(); // match
        let value = self.parse_expr()?;
        self.advance(); // {
        let mut cases = Vec::new();
        let mut default = None;

        while !matches!(self.peek(), TokenKind::Punctuation('}')) {
            match self.peek() {
                TokenKind::Keyword(k) if k == "case" => {
                    self.advance(); // case
                    
                    // Parse pattern which might be Base::Variant or Base::Variant(var)
                    let pat = match self.advance() {
                        TokenKind::Identifier(s) => s.clone(),
                        _ => return None,
                    };
                    
                    // Check for :: operator (enum variant)
                    let pat_str = if matches!(self.peek(), TokenKind::Operator(op) if op == "::") {
                        self.advance(); // ::
                        let variant = match self.advance() {
                            TokenKind::Identifier(v) => v.clone(),
                            _ => return None,
                        };
                        let full_tag = format!("{}::{}", pat, variant);
                        
                        // Check for parameter binding like Result::Ok(msg)
                        if matches!(self.peek(), TokenKind::Punctuation('(')) {
                            self.advance(); // (
                            let var = match self.advance() {
                                TokenKind::Identifier(v) => v.clone(),
                                _ => return None,
                            };
                            self.advance(); // )
                            format!("{}({})", full_tag, var)
                        } else {
                            full_tag
                        }
                    } else {
                        // Plain identifier pattern
                        if matches!(self.peek(), TokenKind::Punctuation('(')) {
                            self.advance(); // (
                            let var = match self.advance() {
                                TokenKind::Identifier(v) => v.clone(),
                                _ => return None,
                            };
                            self.advance(); // )
                            format!("{}({})", pat, var)
                        } else {
                            pat
                        }
                    };
                    
                    self.advance(); // :
                    self.advance(); // {
                    let mut body = Vec::new();
                    while !matches!(self.peek(), TokenKind::Punctuation('}')) {
                        if let Some(stmt) = self.parse_stmt() {
                            body.push(stmt);
                        } else {
                            break;
                        }
                    }
                    self.advance(); // }
                    cases.push((pat_str, body));
                }
                TokenKind::Keyword(k) if k == "default" => {
                    self.advance(); // default
                    self.advance(); // :
                    self.advance(); // {
                    let mut body = Vec::new();
                    while !matches!(self.peek(), TokenKind::Punctuation('}')) {
                        if let Some(stmt) = self.parse_stmt() {
                            body.push(stmt);
                        } else {
                            break;
                        }
                    }
                    self.advance(); // }
                    default = Some(body);
                }
                _ => break,
            }
        }

        self.advance(); // }
        Some(Stmt::Match { value, cases, default })
    }

    fn parse_loop(&mut self) -> Option<Stmt> {
        self.advance(); // loop
        let condition = if matches!(self.peek(), TokenKind::Keyword(k) if k == "while") {
            self.advance(); // while
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.advance(); // {
        let mut body = Vec::new();
        while !matches!(self.peek(), TokenKind::Punctuation('}')) {
            if let Some(stmt) = self.parse_stmt() {
                body.push(stmt);
            } else {
                break;
            }
        }
        self.advance(); // }
        Some(Stmt::Loop { condition, body })
    }

    fn parse_for(&mut self) -> Option<Stmt> {
        self.advance(); // for
        let var = match self.advance() {
            TokenKind::Identifier(v) => v.clone(),
            _ => return None,
        };
        self.advance(); // in
        // Use parse_primary to get just the identifier without postfix operations
        // This prevents "for i in arr { }" from being parsed as struct instantiation
        let iterable = self.parse_primary()?;
        self.advance(); // {
        let mut body = Vec::new();
        while !matches!(self.peek(), TokenKind::Punctuation('}')) {
            if let Some(stmt) = self.parse_stmt() {
                body.push(stmt);
            } else {
                break;
            }
        }
        self.advance(); // }
        Some(Stmt::For { var, iterable, body })
    }

    fn parse_try_except(&mut self) -> Option<Stmt> {
        self.advance(); // try
        self.advance(); // {
        let mut try_block = Vec::new();
        while !matches!(self.peek(), TokenKind::Punctuation('}')) {
            if let Some(stmt) = self.parse_stmt() {
                try_block.push(stmt);
            } else {
                break;
            }
        }
        self.advance(); // }
        self.advance(); // except
        let except_var = match self.advance() {
            TokenKind::Identifier(v) => v.clone(),
            _ => return None,
        };
        self.advance(); // {
        let mut except_block = Vec::new();
        while !matches!(self.peek(), TokenKind::Punctuation('}')) {
            if let Some(stmt) = self.parse_stmt() {
                except_block.push(stmt);
            } else {
                break;
            }
        }
        self.advance(); // }
        Some(Stmt::TryExcept { try_block, except_var, except_block })
    }

    fn parse_import(&mut self) -> Option<Stmt> {
        // Two forms:
        // 1. import module
        // 2. from module import symbol1, symbol2
        
        let is_from = matches!(self.peek(), TokenKind::Keyword(k) if k == "from");
        self.advance(); // import or from
        
        if is_from {
            // from module import ...
            let module = match self.advance() {
                TokenKind::Identifier(m) => m.clone(),
                _ => return None,
            };
            
            // expect 'import' keyword
            if !matches!(self.peek(), TokenKind::Keyword(k) if k == "import") {
                return None;
            }
            self.advance(); // import
            
            // Parse symbol list
            let mut symbols = Vec::new();
            loop {
                match self.advance() {
                    TokenKind::Identifier(s) => symbols.push(s.clone()),
                    _ => return None,
                }
                
                if matches!(self.peek(), TokenKind::Punctuation(',')) {
                    self.advance(); // ,
                } else {
                    break;
                }
            }
            
            Some(Stmt::Import {
                module,
                symbols: Some(symbols),
            })
        } else {
            // import module
            let module = match self.advance() {
                TokenKind::Identifier(m) => m.clone(),
                _ => return None,
            };
            
            Some(Stmt::Import {
                module,
                symbols: None,
            })
        }
    }

    fn parse_export(&mut self) -> Option<Stmt> {
        self.advance(); // export
        
        // Parse the statement to be exported
        let stmt = self.parse_stmt()?;
        
        Some(Stmt::Export {
            stmt: Box::new(stmt),
        })
    }

    fn parse_if(&mut self) -> Option<Stmt> {
        self.advance(); // if
        let condition = self.parse_expr()?;
        self.advance(); // {
        let mut then_branch = Vec::new();
        while !matches!(self.peek(), TokenKind::Punctuation('}')) {
            if let Some(stmt) = self.parse_stmt() {
                then_branch.push(stmt);
            } else {
                break;
            }
        }
        self.advance(); // }
        
        let else_branch = if matches!(self.peek(), TokenKind::Keyword(k) if k == "else") {
            self.advance(); // else
            self.advance(); // {
            let mut else_stmts = Vec::new();
            while !matches!(self.peek(), TokenKind::Punctuation('}')) {
                if let Some(stmt) = self.parse_stmt() {
                    else_stmts.push(stmt);
                } else {
                    break;
                }
            }
            self.advance(); // }
            Some(else_stmts)
        } else {
            None
        };
        
        Some(Stmt::If { condition, then_branch, else_branch })
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        // Check for enum tag (e.g., Result::Ok(...))
        if let TokenKind::Identifier(a) = self.peek() {
            if self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Operator("::".into())) {
                let base = a.clone();
                self.advance(); // base
                self.advance(); // ::
                let variant = match self.advance() {
                    TokenKind::Identifier(v) => v.clone(),
                    _ => return None,
                };
                let mut args = Vec::new();
                if matches!(self.peek(), TokenKind::Punctuation('(')) {
                    self.advance();
                    while !matches!(self.peek(), TokenKind::Punctuation(')')) {
                        if let Some(arg) = self.parse_expr() {
                            args.push(arg);
                        }
                        if matches!(self.peek(), TokenKind::Punctuation(',')) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.advance();
                }
                return Some(Expr::Tag(format!("{}::{}", base, variant), args));
            }
        }

        // Check for built-in functions like print(...) or throw(...)
        if let TokenKind::Identifier(name) = self.peek() {
            let name_clone = name.clone();
            if matches!(name_clone.as_str(), "print" | "throw") {
                if self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Punctuation('(')) {
                    self.advance(); // name
                    self.advance(); // (
                    let mut args = Vec::new();
                    while !matches!(self.peek(), TokenKind::Punctuation(')')) {
                        if let Some(arg) = self.parse_expr() {
                            args.push(arg);
                        }
                        if matches!(self.peek(), TokenKind::Punctuation(',')) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.advance(); // )
                    return Some(Expr::Tag(name_clone, args));
                }
            }
        }

        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Option<Expr> {
        let mut left = self.parse_additive()?;
        
        while matches!(self.peek(), TokenKind::Operator(op) if matches!(op.as_str(), "==" | ">" | "<" | ">=" | "<=")) {
            let op = match self.advance() {
                TokenKind::Operator(o) => o.clone(),
                _ => break,
            };
            let right = self.parse_additive()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Some(left)
    }

    fn parse_additive(&mut self) -> Option<Expr> {
        let mut left = self.parse_multiplicative()?;
        
        while matches!(self.peek(), TokenKind::Operator(op) if matches!(op.as_str(), "+" | "-")) {
            let op = match self.advance() {
                TokenKind::Operator(o) => o.clone(),
                _ => break,
            };
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Some(left)
    }

    fn parse_multiplicative(&mut self) -> Option<Expr> {
        let mut left = self.parse_call()?;
        
        while matches!(self.peek(), TokenKind::Operator(op) if matches!(op.as_str(), "*" | "/")) {
            let op = match self.advance() {
                TokenKind::Operator(o) => o.clone(),
                _ => break,
            };
            let right = self.parse_call()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        
        Some(left)
    }

    fn parse_call(&mut self) -> Option<Expr> {
        let mut expr = self.parse_primary()?;
        
        loop {
            match self.peek() {
                // Handle function calls
                TokenKind::Punctuation('(') => {
                    self.advance(); // (
                    let mut args = Vec::new();
                    while !matches!(self.peek(), TokenKind::Punctuation(')')) {
                        if let Some(arg) = self.parse_expr() {
                            args.push(arg);
                        }
                        if matches!(self.peek(), TokenKind::Punctuation(',')) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.advance(); // )
                    expr = Expr::Call {
                        function: Box::new(expr),
                        args,
                    };
                }
                // Handle field access
                TokenKind::Punctuation('.') => {
                    self.advance(); // .
                    if let TokenKind::Identifier(field) = self.peek() {
                        let field_name = field.clone();
                        self.advance();
                        expr = Expr::FieldAccess {
                            object: Box::new(expr),
                            field: field_name,
                        };
                    } else {
                        break;
                    }
                }
                // Handle index access: arr[index]
                TokenKind::Punctuation('[') => {
                    self.advance(); // [
                    if let Some(index) = self.parse_expr() {
                        if matches!(self.peek(), TokenKind::Punctuation(']')) {
                            self.advance(); // ]
                            expr = Expr::IndexAccess {
                                object: Box::new(expr),
                                index: Box::new(index),
                            };
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                // Handle struct instantiation: Struct { field1: val1, field2: val2 }
                TokenKind::Punctuation('{') if matches!(expr, Expr::Identifier(_)) => {
                    // Only treat as struct instantiation if we have an identifier followed by {
                    // AND there's actually struct field syntax inside (field: value)
                    // Check if next token looks like a field (identifier followed by colon or closing brace)
                    let next_token = self.tokens.get(self.pos + 1);
                    let is_struct = match next_token.map(|t| &t.kind) {
                        Some(TokenKind::Identifier(_)) => {
                            // Check if there's a colon after the identifier
                            self.tokens.get(self.pos + 2).map(|t| &t.kind) == Some(&TokenKind::Punctuation(':'))
                        }
                        Some(TokenKind::Punctuation('}')) => {
                            // Empty braces {} - treat as empty struct
                            true
                        }
                        _ => false
                    };
                    
                    if !is_struct {
                        // Not a struct instantiation, stop parsing here
                        break;
                    }
                    
                    // This is a struct instantiation
                    if let Expr::Identifier(name) = expr {
                        self.advance(); // {
                        let mut fields = Vec::new();
                        
                        loop {
                            if matches!(self.peek(), TokenKind::Punctuation('}')) {
                                break;
                            }
                            
                            if let TokenKind::Identifier(field_name) = self.peek() {
                                let field_name = field_name.clone();
                                self.advance(); // consume field name
                                
                                // Expect colon
                                if !matches!(self.peek(), TokenKind::Punctuation(':')) {
                                    break; // Invalid syntax
                                }
                                self.advance(); // consume :
                                
                                // Parse field value - use parse_primary to avoid recursion
                                // This means field values can only be literals or identifiers for now
                                // TODO: Support full expressions in struct literals
                                if let Some(value) = self.parse_primary() {
                                    fields.push((field_name, value));
                                }
                                
                                // Check for comma or end
                                if matches!(self.peek(), TokenKind::Punctuation(',')) {
                                    self.advance();
                                } else if matches!(self.peek(), TokenKind::Punctuation('}')) {
                                    break;
                                } else {
                                    // Unexpected token, try to recover
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        
                        self.advance(); // consume }
                        expr = Expr::StructInstance { name, fields };
                    }
                    break;
                }
                _ => break,
            }
        }
        
        Some(expr)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match self.peek() {
            TokenKind::Punctuation('[') => self.parse_array_literal(),
            TokenKind::Punctuation('{') => self.parse_dict_literal(),
            _ => {
                // For other tokens, advance and match
                match self.advance() {
                    TokenKind::Identifier(name) => Some(Expr::Identifier(name.clone())),
                    TokenKind::Number(n) => Some(Expr::Number(*n)),
                    TokenKind::String(s) => Some(Expr::String(s.clone())),
                    TokenKind::Bool(b) => Some(Expr::Bool(*b)),
                    _ => None,
                }
            }
        }
    }

    fn parse_array_literal(&mut self) -> Option<Expr> {
        self.advance(); // consume [
        let mut elements = Vec::new();
        
        while !matches!(self.peek(), TokenKind::Punctuation(']')) && !matches!(self.peek(), TokenKind::Eof) {
            // Use parse_comparison to avoid infinite recursion through parse_call
            if let Some(elem) = self.parse_comparison() {
                elements.push(elem);
            }
            
            if matches!(self.peek(), TokenKind::Punctuation(',')) {
                self.advance();
            } else if !matches!(self.peek(), TokenKind::Punctuation(']')) {
                break; // Unexpected token
            }
        }
        
        if matches!(self.peek(), TokenKind::Punctuation(']')) {
            self.advance(); // consume ]
        }
        
        Some(Expr::ArrayLiteral(elements))
    }

    fn parse_dict_literal(&mut self) -> Option<Expr> {
        self.advance(); // consume {
        let mut pairs = Vec::new();
        
        while !matches!(self.peek(), TokenKind::Punctuation('}')) && !matches!(self.peek(), TokenKind::Eof) {
            // Parse key - use parse_comparison to avoid recursion
            let key = self.parse_comparison()?;
            
            // Expect colon
            if !matches!(self.peek(), TokenKind::Punctuation(':')) {
                break;
            }
            self.advance(); // consume :
            
            // Parse value - use parse_comparison to avoid recursion
            let value = self.parse_comparison()?;
            pairs.push((key, value));
            
            if matches!(self.peek(), TokenKind::Punctuation(',')) {
                self.advance();
            } else if !matches!(self.peek(), TokenKind::Punctuation('}')) {
                break;
            }
        }
        
        if matches!(self.peek(), TokenKind::Punctuation('}')) {
            self.advance(); // consume }
        }
        
        Some(Expr::DictLiteral(pairs))
    }

    /// Parse a type annotation (: type_name)
    /// Returns Some(TypeAnnotation) if a type annotation is present, None otherwise
    fn parse_type_annotation(&mut self) -> Option<crate::ast::TypeAnnotation> {
        use crate::ast::TypeAnnotation;
        
        // Check if there's a colon for type annotation
        // But NOT if it's := (assignment operator)
        if !matches!(self.peek(), TokenKind::Punctuation(':')) {
            return None;
        }
        
        // Peek ahead - if next token is '=', this is ':=' not a type annotation
        let saved_pos = self.pos;
        self.advance(); // tentatively consume :
        
        // Check if this is actually part of ':='
        if matches!(self.peek(), TokenKind::Operator(op) if op == "=") {
            // This was part of ':=', backtrack
            self.pos = saved_pos;
            return None;
        }
        
        // Parse the type keyword
        match self.peek() {
            TokenKind::Keyword(k) if k == "int" => {
                self.advance();
                Some(TypeAnnotation::Int)
            }
            TokenKind::Keyword(k) if k == "float" => {
                self.advance();
                Some(TypeAnnotation::Float)
            }
            TokenKind::Keyword(k) if k == "string" => {
                self.advance();
                Some(TypeAnnotation::String)
            }
            TokenKind::Keyword(k) if k == "bool" => {
                self.advance();
                Some(TypeAnnotation::Bool)
            }
            _ => {
                // Invalid type, backtrack
                self.pos = saved_pos;
                None
            }
        }
    }

    // --- TEST RUNNER ---

    pub fn run_all_tests(test_dir: &Path, update_snapshots: bool) {
        let Ok(entries) = fs::read_dir(test_dir) else {
            eprintln!("[!] Failed to read test directory: {}", test_dir.display());
            return;
        };

        let mut passed = 0;
        let mut total = 0;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "ruff").unwrap_or(false) {
                total += 1;
                let content = fs::read_to_string(&path).unwrap_or_default();
                let expected_path = path.with_extension("out");

                let tokens = crate::lexer::tokenize(&content);
                let mut parser = crate::parser::Parser::new(tokens);
                let ast = parser.parse();
                let mut interp = crate::interpreter::Interpreter::new();
                interp.set_source(path.to_string_lossy().to_string(), &content);

                let start = Instant::now();

                let buffer = Arc::new(Mutex::new(Vec::new()));
                let _ = interp.set_output(buffer.clone());

                interp.eval_stmts(&ast);

                let actual = {
                    let lock = buffer.lock().unwrap();
                    String::from_utf8_lossy(&lock).trim().to_string()
                };

                let expected = if expected_path.exists() && !update_snapshots {
                    fs::read_to_string(&expected_path).unwrap_or_default().trim().to_string()
                } else {
                    fs::write(&expected_path, &actual).ok();
                    actual.clone()
                };

                if actual == expected {
                    println!("[✓] {} ({:.2?})", path.display(), start.elapsed());
                    passed += 1;
                } else {
                    println!("[✗] {}", path.display());
                    println!("Expected:\n{}\nGot:\n{}\n", expected, actual);
                }
            }
        }

        println!("\n[✓] Passed {}/{} tests", passed, total);
    }
}
