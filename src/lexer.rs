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
// - Single-line comments starting with #
// - Multi-line comments /* ... */
// - Doc comments starting with ///

#[derive(Debug, Clone, PartialEq)] // Added PartialEq here
pub enum TokenKind {
    Identifier(String),
    Number(f64),
    String(String),
    InterpolatedString(Vec<InterpolatedPart>), // String with ${} expressions
    Bool(bool),
    Operator(String),
    Punctuation(char),
    Keyword(String),
    Eof,
}

/// Represents parts of an interpolated string
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolatedPart {
    Text(String),       // Regular text
    Expression(String), // Expression inside ${}
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
                chars.next(); // consume #
                col += 1;

                // Regular single-line comment
                while let Some(&ch) = chars.peek() {
                    chars.next();
                    if ch == '\n' {
                        line += 1;
                        col = 1;
                        break;
                    }
                    col += 1;
                }
            }
            '"' => {
                chars.next(); // skip opening quote
                let mut parts = Vec::new();
                let mut current_text = String::new();
                let mut has_interpolation = false;

                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        chars.next();
                        break;
                    }

                    if ch == '\\' {
                        chars.next();
                        if let Some(&esc) = chars.peek() {
                            chars.next();
                            match esc {
                                'n' => current_text.push('\n'),
                                't' => current_text.push('\t'),
                                '\\' => current_text.push('\\'),
                                '"' => current_text.push('"'),
                                _ => current_text.push(esc),
                            }
                        }
                    } else if ch == '$' {
                        chars.next();
                        if chars.peek() == Some(&'{') {
                            // Found interpolation start
                            has_interpolation = true;
                            chars.next(); // skip {

                            // Save current text as a part
                            if !current_text.is_empty() {
                                parts.push(InterpolatedPart::Text(current_text.clone()));
                                current_text.clear();
                            }

                            // Collect expression until }
                            let mut expr = String::new();
                            let mut brace_depth = 1;
                            while let Some(&ch) = chars.peek() {
                                chars.next();
                                if ch == '{' {
                                    brace_depth += 1;
                                    expr.push(ch);
                                } else if ch == '}' {
                                    brace_depth -= 1;
                                    if brace_depth == 0 {
                                        break;
                                    }
                                    expr.push(ch);
                                } else {
                                    expr.push(ch);
                                }
                            }

                            parts.push(InterpolatedPart::Expression(expr));
                        } else {
                            // Just a $ without {, treat as normal text
                            current_text.push('$');
                        }
                    } else {
                        chars.next();
                        current_text.push(ch);
                    }
                }

                // Add remaining text
                if !current_text.is_empty() {
                    parts.push(InterpolatedPart::Text(current_text));
                }

                // Create appropriate token
                if has_interpolation {
                    tokens.push(Token {
                        kind: TokenKind::InterpolatedString(parts),
                        line,
                        column: col,
                    });
                } else {
                    // No interpolation, use regular string
                    let text = if parts.is_empty() {
                        String::new()
                    } else if let InterpolatedPart::Text(s) = &parts[0] {
                        s.clone()
                    } else {
                        String::new()
                    };
                    tokens.push(Token { kind: TokenKind::String(text), line, column: col });
                }
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
                tokens.push(Token { kind: TokenKind::Number(parsed), line, column: col });
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
                    "let" | "mut" | "const" | "func" | "return" | "enum" | "match" | "case"
                    | "default" | "if" | "else" | "loop" | "while" | "for" | "in" | "break"
                    | "continue" | "try" | "except" | "int" | "float" | "string" | "bool"
                    | "import" | "export" | "from" | "struct" | "impl" | "self" => {
                        TokenKind::Keyword(ident)
                    }
                    "true" => TokenKind::Bool(true),
                    "false" => TokenKind::Bool(false),
                    _ => TokenKind::Identifier(ident),
                };

                tokens.push(Token { kind, line, column: col });
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
                    tokens.push(Token { kind: TokenKind::Punctuation(':'), line, column: col });
                }
            }
            '=' | '+' | '-' | '*' | '<' | '>' | '!' => {
                let op = chars.next().unwrap();
                col += 1;
                // Check for == >= <= -> !=
                if op == '=' && chars.peek() == Some(&'=') {
                    chars.next();
                    col += 1;
                    tokens.push(Token {
                        kind: TokenKind::Operator("==".into()),
                        line,
                        column: col,
                    });
                } else if op == '!' && chars.peek() == Some(&'=') {
                    chars.next();
                    col += 1;
                    tokens.push(Token {
                        kind: TokenKind::Operator("!=".into()),
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
            '/' => {
                chars.next(); // consume /
                col += 1;

                // Check for multi-line comment /* */
                if chars.peek() == Some(&'*') {
                    chars.next(); // consume *
                    col += 1;

                    // Multi-line comment - scan until */
                    let mut found_end = false;
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch == '\n' {
                            line += 1;
                            col = 1;
                        } else {
                            col += 1;
                        }

                        if ch == '*' && chars.peek() == Some(&'/') {
                            chars.next(); // consume /
                            col += 1;
                            found_end = true;
                            break;
                        }
                    }

                    // If we didn't find closing */, that's a syntax error but we'll continue
                    // The parser/interpreter can handle this gracefully
                    if !found_end {
                        // Unterminated multi-line comment
                        // Continue processing but note this could be an error
                    }
                } else if chars.peek() == Some(&'/') {
                    chars.next(); // consume second /
                    col += 1;

                    // Check for doc comment (///)
                    if chars.peek() == Some(&'/') {
                        chars.next(); // consume third /
                        col += 1;

                        // Doc comment - consume until end of line
                        while let Some(&ch) = chars.peek() {
                            chars.next();
                            if ch == '\n' {
                                line += 1;
                                col = 1;
                                break;
                            }
                            col += 1;
                        }
                    } else {
                        // Regular // comment (not standard Ruff, but let's support it)
                        while let Some(&ch) = chars.peek() {
                            chars.next();
                            if ch == '\n' {
                                line += 1;
                                col = 1;
                                break;
                            }
                            col += 1;
                        }
                    }
                } else {
                    // Regular division operator
                    tokens.push(Token { kind: TokenKind::Operator("/".into()), line, column: col });
                }
            }
            '%' => {
                chars.next();
                col += 1;
                tokens.push(Token { kind: TokenKind::Operator("%".into()), line, column: col });
            }
            '|' => {
                chars.next();
                col += 1;
                if chars.peek() == Some(&'|') {
                    chars.next();
                    col += 1;
                    tokens.push(Token {
                        kind: TokenKind::Operator("||".into()),
                        line,
                        column: col,
                    });
                } else {
                    // Single | is not a valid operator in Ruff yet
                    tokens.push(Token { kind: TokenKind::Operator("|".into()), line, column: col });
                }
            }
            '&' => {
                chars.next();
                col += 1;
                if chars.peek() == Some(&'&') {
                    chars.next();
                    col += 1;
                    tokens.push(Token {
                        kind: TokenKind::Operator("&&".into()),
                        line,
                        column: col,
                    });
                } else {
                    // Single & is not a valid operator in Ruff yet
                    tokens.push(Token { kind: TokenKind::Operator("&".into()), line, column: col });
                }
            }
            '(' | ')' | '{' | '}' | '[' | ']' | ',' | ';' | '.' => {
                tokens.push(Token { kind: TokenKind::Punctuation(c), line, column: col });
                chars.next();
                col += 1;
            }
            _ => {
                chars.next();
                col += 1;
            }
        }
    }

    tokens.push(Token { kind: TokenKind::Eof, line, column: col });

    tokens
}
