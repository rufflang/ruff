use crate::lexer::{self, Token, TokenKind};
use crate::parser;
use std::panic::{self, AssertUnwindSafe};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
}

impl DiagnosticSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub severity: DiagnosticSeverity,
    pub message: String,
}

pub fn diagnose(source: &str) -> Vec<Diagnostic> {
    let tokens = lexer::tokenize(source);
    let mut diagnostics = Vec::new();

    diagnostics.extend(check_delimiter_balance(&tokens));
    diagnostics.extend(check_parser_panics(&tokens));

    diagnostics.sort_by_key(|diagnostic| (diagnostic.line, diagnostic.column));
    diagnostics
}

fn check_delimiter_balance(tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut brace_stack: Vec<(usize, usize)> = Vec::new();
    let mut paren_stack: Vec<(usize, usize)> = Vec::new();
    let mut bracket_stack: Vec<(usize, usize)> = Vec::new();

    for token in tokens.iter() {
        match token.kind {
            TokenKind::Punctuation('{') => brace_stack.push((token.line, token.column)),
            TokenKind::Punctuation('}') => {
                if brace_stack.pop().is_none() {
                    diagnostics.push(Diagnostic {
                        line: token.line,
                        column: token.column,
                        severity: DiagnosticSeverity::Error,
                        message: "Unmatched closing brace '}'".to_string(),
                    });
                }
            }
            TokenKind::Punctuation('(') => paren_stack.push((token.line, token.column)),
            TokenKind::Punctuation(')') => {
                if paren_stack.pop().is_none() {
                    diagnostics.push(Diagnostic {
                        line: token.line,
                        column: token.column,
                        severity: DiagnosticSeverity::Error,
                        message: "Unmatched closing parenthesis ')'".to_string(),
                    });
                }
            }
            TokenKind::Punctuation('[') => bracket_stack.push((token.line, token.column)),
            TokenKind::Punctuation(']') => {
                if bracket_stack.pop().is_none() {
                    diagnostics.push(Diagnostic {
                        line: token.line,
                        column: token.column,
                        severity: DiagnosticSeverity::Error,
                        message: "Unmatched closing bracket ']'".to_string(),
                    });
                }
            }
            _ => {}
        }
    }

    for (line, column) in brace_stack.into_iter() {
        diagnostics.push(Diagnostic {
            line,
            column,
            severity: DiagnosticSeverity::Error,
            message: "Unclosed opening brace '{'".to_string(),
        });
    }
    for (line, column) in paren_stack.into_iter() {
        diagnostics.push(Diagnostic {
            line,
            column,
            severity: DiagnosticSeverity::Error,
            message: "Unclosed opening parenthesis '('".to_string(),
        });
    }
    for (line, column) in bracket_stack.into_iter() {
        diagnostics.push(Diagnostic {
            line,
            column,
            severity: DiagnosticSeverity::Error,
            message: "Unclosed opening bracket '['".to_string(),
        });
    }

    diagnostics
}

fn check_parser_panics(tokens: &[Token]) -> Vec<Diagnostic> {
    let mut parser = parser::Parser::new(tokens.to_vec());
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let parse_result = panic::catch_unwind(AssertUnwindSafe(|| parser.parse()));
    panic::set_hook(previous_hook);

    match parse_result {
        Ok(_) => Vec::new(),
        Err(payload) => {
            let message = if let Some(message) = payload.downcast_ref::<String>() {
                message.clone()
            } else if let Some(message) = payload.downcast_ref::<&str>() {
                message.to_string()
            } else {
                "Parser panic while producing diagnostics".to_string()
            };

            vec![Diagnostic {
                line: 1,
                column: 1,
                severity: DiagnosticSeverity::Error,
                message,
            }]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{diagnose, DiagnosticSeverity};

    #[test]
    fn diagnostics_empty_for_valid_program() {
        let source = [
            "func greet(name) {",
            "    return name",
            "}",
            "print(greet(\"ruff\"))",
        ]
        .join("\n");

        let diagnostics = diagnose(&source);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn diagnostics_report_unmatched_closing_brace() {
        let diagnostics = diagnose("}\n");
        assert!(diagnostics.iter().any(|diagnostic| {
            DiagnosticSeverity::Error == diagnostic.severity
                && diagnostic.message.contains("Unmatched closing brace")
        }));
    }

    #[test]
    fn diagnostics_report_unclosed_opening_parenthesis() {
        let diagnostics = diagnose("print((1 + 2)\n");
        assert!(diagnostics.iter().any(|diagnostic| {
            DiagnosticSeverity::Error == diagnostic.severity
                && diagnostic.message.contains("Unclosed opening parenthesis")
        }));
    }

    #[test]
    fn diagnostics_capture_parser_panic_message() {
        let diagnostics = diagnose("let result: Result<int := 1\n");
        assert!(diagnostics.iter().any(|diagnostic| {
            DiagnosticSeverity::Error == diagnostic.severity
                && diagnostic.message.contains("Expected ',' in Result<T, E> type")
        }));
    }
}