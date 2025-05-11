#[derive(Debug, Clone, PartialEq)] // Added PartialEq here
pub enum TokenKind {
    Identifier(String),
    Number(f64),
    String(String),
    Operator(String),
    Punctuation(char),
    Keyword(String),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

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
                    "while" | "for" | "in" | "try" | "except" | "throw" => {
                        TokenKind::Keyword(ident)
                    }
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
                tokens.push(Token {
                    kind: TokenKind::Operator(op.to_string()),
                    line,
                    column: col,
                });
            }
            '(' | ')' | '{' | '}' | ',' | ';' => {
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
