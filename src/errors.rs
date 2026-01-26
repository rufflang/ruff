// File: src/errors.rs
//
// Error handling and reporting for the Ruff programming language.
// Provides structured error types with source location information
// and pretty-printed error messages.

use colored::Colorize;
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

/// A structured error with location information
#[derive(Debug, Clone)]
pub struct RuffError {
    pub kind: ErrorKind,
    pub message: String,
    pub location: SourceLocation,
    pub source_line: Option<String>,
    pub suggestion: Option<String>,
    pub help: Option<String>,
    pub note: Option<String>,
}

#[allow(dead_code)]
impl RuffError {
    pub fn new(kind: ErrorKind, message: String, location: SourceLocation) -> Self {
        Self {
            kind,
            message,
            location,
            source_line: None,
            suggestion: None,
            help: None,
            note: None,
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
        writeln!(f, "{}: {}", kind_str.red().bold(), self.message.bold())?;

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
