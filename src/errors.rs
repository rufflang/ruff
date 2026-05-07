// File: src/errors.rs
//
// Error handling and reporting for the Ruff programming language.
// Provides structured error types with source location information
// and pretty-printed error messages.

use colored::Colorize;
use serde_json::json;
use std::fmt;

/// Source location information for tracking where code appears in a file
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub file: Option<String>,
}

#[allow(dead_code)]
impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column, file: None }
    }

    pub fn with_file(line: usize, column: usize, file: String) -> Self {
        Self { line, column, file: Some(file) }
    }

    pub fn unknown() -> Self {
        Self { line: 0, column: 0, file: None }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref file) = self.file {
            write!(f, "{}:{}:{}", file, self.line, self.column)
        } else {
            write!(f, "{}:{}", self.line, self.column)
        }
    }
}

pub const DIAGNOSTIC_CODE_LEXER: &str = "RUFLEX001";
pub const DIAGNOSTIC_CODE_PARSER: &str = "RUFPARSE001";
pub const DIAGNOSTIC_CODE_RUNTIME: &str = "RUFRUN001";
pub const DIAGNOSTIC_CODE_VM: &str = "RUFVM001";
pub const DIAGNOSTIC_CODE_CLI: &str = "RUFCLI001";
pub const DIAGNOSTIC_CODE_LSP: &str = "RUFLSP001";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSubsystem {
    Lexer,
    Parser,
    Runtime,
    Vm,
    Cli,
    Lsp,
}

impl DiagnosticSubsystem {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticSubsystem::Lexer => "lexer",
            DiagnosticSubsystem::Parser => "parser",
            DiagnosticSubsystem::Runtime => "runtime",
            DiagnosticSubsystem::Vm => "vm",
            DiagnosticSubsystem::Cli => "cli",
            DiagnosticSubsystem::Lsp => "lsp",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub subsystem: DiagnosticSubsystem,
    pub message: String,
    pub help: Option<String>,
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
}

impl Diagnostic {
    pub fn new(
        code: impl Into<String>,
        severity: DiagnosticSeverity,
        subsystem: DiagnosticSubsystem,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            subsystem,
            message: message.into(),
            help: None,
            file: None,
            line: 0,
            column: 0,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_location(
        mut self,
        file: Option<String>,
        line: usize,
        column: usize,
    ) -> Self {
        self.file = file;
        self.line = line;
        self.column = column;
        self
    }

    pub fn render_human(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "[{}] [{}] {}: {}",
            self.code,
            self.subsystem.as_str(),
            self.severity.as_str(),
            self.message
        ));

        if self.line > 0 && self.column > 0 {
            let location = if let Some(file) = &self.file {
                format!("{}:{}:{}", file, self.line, self.column)
            } else {
                format!("{}:{}", self.line, self.column)
            };
            lines.push(format!("  --> {}", location));
        }

        if let Some(help) = &self.help {
            lines.push(format!("  = help: {}", help));
        }

        lines.join("\n")
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        json!({
            "code": self.code,
            "severity": self.severity.as_str(),
            "subsystem": self.subsystem.as_str(),
            "message": self.message,
            "help": self.help,
            "file": self.file,
            "line": self.line,
            "column": self.column,
        })
    }
}

/// Types of errors that can occur in Ruff
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ErrorKind {
    ParseError,
    RuntimeError,
    TypeError,
    UndefinedVariable,
    UndefinedFunction,
    DivisionByZero,
    InvalidOperation,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::ParseError => write!(f, "Parse Error"),
            ErrorKind::RuntimeError => write!(f, "Runtime Error"),
            ErrorKind::TypeError => write!(f, "Type Error"),
            ErrorKind::UndefinedVariable => write!(f, "Undefined Variable"),
            ErrorKind::UndefinedFunction => write!(f, "Undefined Function"),
            ErrorKind::DivisionByZero => write!(f, "Division By Zero"),
            ErrorKind::InvalidOperation => write!(f, "Invalid Operation"),
        }
    }
}

/// A structured error with location information and call stack
#[derive(Debug, Clone)]
pub struct RuffError {
    pub kind: ErrorKind,
    pub diagnostic_code: String,
    pub subsystem: DiagnosticSubsystem,
    pub message: String,
    pub location: SourceLocation,
    pub source_line: Option<String>,
    pub suggestion: Option<String>,
    pub help: Option<String>,
    pub note: Option<String>,
    pub call_stack: Vec<String>,
}

#[allow(dead_code)]
impl RuffError {
    fn default_diagnostic_for_kind(kind: &ErrorKind) -> (&'static str, DiagnosticSubsystem) {
        match kind {
            ErrorKind::ParseError => (DIAGNOSTIC_CODE_PARSER, DiagnosticSubsystem::Parser),
            ErrorKind::RuntimeError
            | ErrorKind::TypeError
            | ErrorKind::UndefinedVariable
            | ErrorKind::UndefinedFunction
            | ErrorKind::DivisionByZero
            | ErrorKind::InvalidOperation => {
                (DIAGNOSTIC_CODE_RUNTIME, DiagnosticSubsystem::Runtime)
            }
        }
    }

    pub fn new(kind: ErrorKind, message: String, location: SourceLocation) -> Self {
        let (diagnostic_code, subsystem) = Self::default_diagnostic_for_kind(&kind);
        Self {
            kind,
            diagnostic_code: diagnostic_code.to_string(),
            subsystem,
            message,
            location,
            source_line: None,
            suggestion: None,
            help: None,
            note: None,
            call_stack: Vec::new(),
        }
    }

    pub fn with_source(mut self, source_line: String) -> Self {
        self.source_line = Some(source_line);
        self
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }

    pub fn with_call_stack(mut self, call_stack: Vec<String>) -> Self {
        self.call_stack = call_stack;
        self
    }

    pub fn with_diagnostic_code(mut self, diagnostic_code: impl Into<String>) -> Self {
        self.diagnostic_code = diagnostic_code.into();
        self
    }

    pub fn with_subsystem(mut self, subsystem: DiagnosticSubsystem) -> Self {
        self.subsystem = subsystem;
        self
    }

    pub fn as_diagnostic(&self) -> Diagnostic {
        let mut diagnostic = Diagnostic::new(
            self.diagnostic_code.clone(),
            DiagnosticSeverity::Error,
            self.subsystem,
            self.message.clone(),
        );
        if self.location.line > 0 && self.location.column > 0 {
            diagnostic = diagnostic.with_location(
                self.location.file.clone(),
                self.location.line,
                self.location.column,
            );
        }
        if let Some(help) = &self.help {
            diagnostic = diagnostic.with_help(help.clone());
        }
        diagnostic
    }

    /// Create a parse error
    pub fn parse_error(message: String, location: SourceLocation) -> Self {
        Self::new(ErrorKind::ParseError, message, location)
    }

    /// Create a runtime error
    pub fn runtime_error(message: String, location: SourceLocation) -> Self {
        Self::new(ErrorKind::RuntimeError, message, location)
    }

    /// Create an undefined variable error
    pub fn undefined_variable(name: String, location: SourceLocation) -> Self {
        Self::new(
            ErrorKind::UndefinedVariable,
            format!("Variable '{}' is not defined", name),
            location,
        )
    }

    /// Create an undefined function error
    pub fn undefined_function(name: String, location: SourceLocation) -> Self {
        Self::new(
            ErrorKind::UndefinedFunction,
            format!("Function '{}' is not defined", name),
            location,
        )
    }
}

impl fmt::Display for RuffError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Error header with kind and message
        let kind_str = format!("{}", self.kind);
        let code_str = format!("[{}]", self.diagnostic_code).bright_magenta().bold();
        let subsystem = format!("[{}]", self.subsystem.as_str()).bright_cyan();
        writeln!(f, "{} {} {}: {}", code_str, subsystem, kind_str.red().bold(), self.message.bold())?;

        // Location arrow
        let location_str = format!("  --> {}", self.location);
        writeln!(f, "{}", location_str.bright_blue())?;

        // Source code context
        if let Some(ref source) = self.source_line {
            let line_num = self.location.line;
            let col_num = self.location.column;

            writeln!(f, "   {}", "|".bright_blue())?;
            writeln!(
                f,
                "{} {} {}",
                format!("{:3}", line_num).bright_blue(),
                "|".bright_blue(),
                source
            )?;
            writeln!(
                f,
                "   {} {}{}",
                "|".bright_blue(),
                " ".repeat(col_num.saturating_sub(1)),
                "^".red().bold()
            )?;
            writeln!(f, "   {}", "|".bright_blue())?;
        }

        // Additional context sections
        if let Some(ref help) = self.help {
            writeln!(
                f,
                "   {} {}",
                "=".bright_yellow(),
                format!("help: {}", help).bright_yellow()
            )?;
        }

        if let Some(ref suggestion) = self.suggestion {
            writeln!(
                f,
                "   {} {}",
                "=".bright_green(),
                format!("Did you mean '{}'?", suggestion).bright_green()
            )?;
        }

        if let Some(ref note) = self.note {
            writeln!(f, "   {} {}", "=".bright_cyan(), format!("note: {}", note).bright_cyan())?;
        }

        // Call stack trace
        if !self.call_stack.is_empty() {
            writeln!(f)?;
            writeln!(f, "{}", "Call stack:".bright_white().bold())?;
            for (i, frame) in self.call_stack.iter().rev().enumerate() {
                writeln!(f, "  {} at {}", format!("{}", i).bright_blue(), frame.bright_white())?;
            }
        }

        Ok(())
    }
}

/// Computes the Levenshtein distance between two strings
/// Used for "Did you mean?" suggestions
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first column and row
    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Compute distances
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    matrix[len1][len2]
}

/// Find the closest match from a list of candidates using Levenshtein distance
/// Returns None if no good match is found (distance > 3)
pub fn find_closest_match<'a>(target: &str, candidates: &'a [String]) -> Option<&'a str> {
    if candidates.is_empty() {
        return None;
    }

    let mut best_match = None;
    let mut best_distance = usize::MAX;

    for candidate in candidates {
        let distance = levenshtein_distance(target, candidate);

        // Only consider reasonably close matches (distance <= 3)
        // and prefer shorter distances
        if distance <= 3 && distance < best_distance {
            best_distance = distance;
            best_match = Some(candidate.as_str());
        }
    }

    best_match
}

impl std::error::Error for RuffError {}
