// File: src/errors.rs
//
// Error handling and reporting for the Ruff programming language.
// Provides structured error types with source location information
// and pretty-printed error messages.

use std::fmt;
use colored::Colorize;

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
        Self {
            line,
            column,
            file: None,
        }
    }

    pub fn with_file(line: usize, column: usize, file: String) -> Self {
        Self {
            line,
            column,
            file: Some(file),
        }
    }

    pub fn unknown() -> Self {
        Self {
            line: 0,
            column: 0,
            file: None,
        }
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
}

#[allow(dead_code)]
impl RuffError {
    pub fn new(kind: ErrorKind, message: String, location: SourceLocation) -> Self {
        Self {
            kind,
            message,
            location,
            source_line: None,
        }
    }

    pub fn with_source(mut self, source_line: String) -> Self {
        self.source_line = Some(source_line);
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
            writeln!(f, "{} {} {}", 
                format!("{:3}", line_num).bright_blue(), 
                "|".bright_blue(),
                source
            )?;
            writeln!(f, "   {} {}{}", 
                "|".bright_blue(),
                " ".repeat(col_num.saturating_sub(1)), 
                "^".red().bold()
            )?;
        }
        
        Ok(())
    }
}

impl std::error::Error for RuffError {}
