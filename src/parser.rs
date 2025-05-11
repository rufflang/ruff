// File: src/parser.rs

use crate::ast::{Expr, Stmt};
use crate::lexer::{Token, TokenKind};
use std::fs;
use std::path::Path;
use std::time::Instant;
use std::sync::{Arc, Mutex};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    fn advance(&mut self) -> &TokenKind {
        let tok = self.tokens.get(self.pos).map(|t| &t.kind).unwrap_or(&TokenKind::Eof);
        self.pos += 1;
        tok
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !matches!(self.peek(), TokenKind::Eof) {
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
            TokenKind::Keyword(k) if k == "return" => {
                self.advance();
                let expr = if !matches!(self.peek(), TokenKind::Punctuation(';')) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                Some(Stmt::Return(expr))
            }
            TokenKind::Keyword(k) if k == "try" => self.parse_try_except(),
            TokenKind::Keyword(k) if k == "match" => self.parse_match(),
            TokenKind::Keyword(k) if k == "loop" => self.parse_loop(),
            TokenKind::Keyword(k) if k == "for" => self.parse_for(),
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

    fn parse_let(&mut self) -> Option<Stmt> {
        let is_mut = matches!(self.advance(), TokenKind::Keyword(k) if k == "mut");
        let name = match self.advance() {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        self.advance(); // :=
        let value = self.parse_expr()?;
        Some(Stmt::Let {
            name,
            value,
            mutable: is_mut,
        })
    }

    fn parse_const(&mut self) -> Option<Stmt> {
        self.advance(); // const
        let name = match self.advance() {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        self.advance(); // :=
        let value = self.parse_expr()?;
        Some(Stmt::Const { name, value })
    }

    fn parse_func(&mut self) -> Option<Stmt> {
        self.advance(); // func
        let name = match self.advance() {
            TokenKind::Identifier(n) => n.clone(),
            _ => return None,
        };
        self.advance(); // (
        let mut params = Vec::new();
        while let TokenKind::Identifier(p) = self.peek() {
            params.push(p.clone());
            self.advance();
            if matches!(self.peek(), TokenKind::Punctuation(',')) {
                self.advance();
            } else {
                break;
            }
        }
        self.advance(); // )
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
        Some(Stmt::FuncDef { name, params, body })
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
                    let pat = match self.advance() {
                        TokenKind::Identifier(s) => s.clone(),
                        _ => return None,
                    };
                    let pat_str = if matches!(self.peek(), TokenKind::Punctuation('(')) {
                        self.advance(); // (
                        let var = match self.advance() {
                            TokenKind::Identifier(v) => v.clone(),
                            _ => return None,
                        };
                        self.advance(); // )
                        format!("{}({})", pat, var)
                    } else {
                        pat
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
        let iterable = self.parse_expr()?;
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

    fn parse_expr(&mut self) -> Option<Expr> {
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

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        match self.advance() {
            TokenKind::Identifier(name) => Some(Expr::Identifier(name.clone())),
            TokenKind::Number(n) => Some(Expr::Number(*n)),
            TokenKind::String(s) => Some(Expr::String(s.clone())),
            _ => None,
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
