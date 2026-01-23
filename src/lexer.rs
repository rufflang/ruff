// File: src/lexer.rs
//
// Lexical analyzer (tokenizer) for the Ruff programming language.
// Converts source code text into a stream of tokens for parsing.
//
// Supports:
// - Keywords: let, mut, const, func, return, enum, match, case, if, else, loop, for, try, except, int, float, string, bool, import, export, from
// - Identifiers and numbers
// - String literals with escape sequences
// - Operators: +, -, *, /, =, ==, <, >, <=, >=, ->, :=, ::
// - Punctuation: ( ) { } , ; :
// - Comments starting with #

#[derive(Debug, Clone, PartialEq)] // Added PartialEq here
pub enum TokenKind {
    Identifier(String),
    Number(f64),
    String(String),
    Bool(bool),
    Operator(String),
    Punctuation(char),
    Keyword(String),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    #[allow(dead_code)]
    pub line: usize,
    #[allow(dead_code)]
    pub column: usize,
}

/// Tokenizes Ruff source code into a vector of tokens.
///
/// Processes the input character by character, recognizing keywords, identifiers,
/// numbers, strings, operators, and punctuation. Comments starting with # are
/// skipped until end of line.
///
/// # Arguments
/// * `source` - The Ruff source code as a string
///
/// # Returns
/// A vector of Token structs representing the tokenized source code
pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    let mut line = 1;
    let mut col = 1;

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => {
                chars.next();
                col += 1;
            }
            '\n' => {
                chars.next();
                line += 1;
                col = 1;
            }
            '#' => {
                while let Some(&ch) = chars.peek() {
                    chars.next();
                    if ch == '\n' {
                        line += 1;
                        col = 1;
                        break;
                    }
                }
            }
            '"' => {
                chars.next(); // skip quote
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    chars.next();
                    if ch == '"' {
                        break;
                    }
                    if ch == '\\' {
                        if let Some(&esc) = chars.peek() {
                            chars.next();
                            match esc {
                                'n' => s.push('\n'),
                                't' => s.push('\t'),
                                '\\' => s.push('\\'),
                                '"' => s.push('"'),
                                _ => s.push(esc),
                            }
                        }
                    } else {
                        s.push(ch);
                    }
                }
                tokens.push(Token {
                    kind: TokenKind::String(s),
                    line,
                    column: col,
                });
                col += 1;
            }
            '0'..='9' => {
                let mut num = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() || ch == '.' {
                        num.push(ch);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }
                let parsed = num.parse().unwrap_or(0.0);
                tokens.push(Token {
                    kind: TokenKind::Number(parsed),
                    line,
                    column: col,
                });
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }

                let kind = match ident.as_str() {
                    "let" | "mut" | "const" | "func" | "return" | "enum" |
                    "match" | "case" | "default" | "if" | "else" | "loop" |
                    "while" | "for" | "in" | "break" | "continue" | "try" | "except" |
                    "int" | "float" | "string" | "bool" |
                    "import" | "export" | "from" | "struct" | "impl" | "self" => {
                        TokenKind::Keyword(ident)
                    }
                    "true" => TokenKind::Bool(true),
                    "false" => TokenKind::Bool(false),
                    _ => TokenKind::Identifier(ident),
                };

                tokens.push(Token {
                    kind,
                    line,
                    column: col,
                });
            }
            ':' => {
                chars.next();
                col += 1;
                if chars.peek() == Some(&'=') {
                    chars.next();
                    col += 1;
                    tokens.push(Token {
                        kind: TokenKind::Operator(":=".into()),
                        line,
                        column: col,
                    });
                } else if chars.peek() == Some(&':') {
                    chars.next();
                    col += 1;
                    tokens.push(Token {
                        kind: TokenKind::Operator("::".into()),
                        line,
                        column: col,
                    });
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Punctuation(':'),
                        line,
                        column: col,
                    });
                }
            }
            '=' | '+' | '-' | '*' | '/' | '<' | '>' => {
                let op = chars.next().unwrap();
                col += 1;
                // Check for == >= <= ->
                if op == '=' && chars.peek() == Some(&'=') {
                    chars.next();
                    col += 1;
                    tokens.push(Token {
                        kind: TokenKind::Operator("==".into()),
                        line,
                        column: col,
                    });
                } else if op == '>' && chars.peek() == Some(&'=') {
                    chars.next();
                    col += 1;
                    tokens.push(Token {
                        kind: TokenKind::Operator(">=".into()),
                        line,
                        column: col,
                    });
                } else if op == '<' && chars.peek() == Some(&'=') {
                    chars.next();
                    col += 1;
                    tokens.push(Token {
                        kind: TokenKind::Operator("<=".into()),
                        line,
                        column: col,
                    });
                } else if op == '-' && chars.peek() == Some(&'>') {
                    chars.next();
                    col += 1;
                    tokens.push(Token {
                        kind: TokenKind::Operator("->".into()),
                        line,
                        column: col,
                    });
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Operator(op.to_string()),
                        line,
                        column: col,
                    });
                }
            }
            '(' | ')' | '{' | '}' | '[' | ']' | ',' | ';' | '.' => {
                tokens.push(Token {
                    kind: TokenKind::Punctuation(c),
                    line,
                    column: col,
                });
                chars.next();
                col += 1;
            }
            _ => {
                chars.next();
                col += 1;
            }
        }
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        line,
        column: col,
    });

    tokens
}
