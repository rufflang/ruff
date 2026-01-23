// File: src/repl.rs
//
// Interactive REPL (Read-Eval-Print Loop) for the Ruff programming language.
// Provides an interactive shell for executing Ruff code with features like:
// - Multi-line input support for functions, loops, and control structures
// - Command history with up/down arrow navigation
// - Line editing capabilities
// - Special commands (:help, :clear, :quit, :vars)
// - Persistent state across inputs
// - Proper error handling and display

use crate::ast::Stmt;
use crate::interpreter::{Interpreter, Value};
use crate::lexer;
use crate::parser;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::HashMap;

/// REPL session that maintains interpreter state and handles user interaction
pub struct Repl {
    interpreter: Interpreter,
    editor: DefaultEditor,
}

impl Repl {
    /// Creates a new REPL session with a fresh interpreter
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let editor = DefaultEditor::new()?;
        Ok(Repl { interpreter: Interpreter::new(), editor })
    }

    /// Displays the welcome banner with version and help information
    fn show_banner(&self) {
        println!("{}", "╔══════════════════════════════════════════════════════╗".bright_cyan());
        println!(
            "{}",
            "║          Ruff REPL v0.5.0 - Interactive Shell       ║".bright_cyan()
        );
        println!("{}", "╚══════════════════════════════════════════════════════╝".bright_cyan());
        println!();
        println!("  {} Use {}{}{}{}",
            "Welcome!".bright_green(),
            ":".bright_blue(),
            "help".bright_yellow(),
            " for commands or ".bright_blue(),
            ":quit".bright_yellow()
        );
        println!("  {} Multi-line input: End with unclosed braces", "Tip:".bright_magenta());
        println!();
    }

    /// Starts the REPL loop
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.show_banner();

        let mut buffer = String::new();

        loop {
            // Determine prompt based on whether we're in multi-line mode
            let prompt = if buffer.is_empty() {
                "ruff> ".bright_green().to_string()
            } else {
                "....> ".bright_blue().to_string()
            };

            match self.editor.readline(&prompt) {
                Ok(line) => {
                    // Add to history
                    let _ = self.editor.add_history_entry(line.as_str());

                    // Check for special commands (only when not in multi-line mode)
                    if buffer.is_empty() && line.trim().starts_with(':') {
                        if self.handle_command(&line.trim()) {
                            continue;
                        } else {
                            break; // :quit was called
                        }
                    }

                    // Accumulate input
                    buffer.push_str(&line);
                    buffer.push('\n');

                    // Check if input is complete
                    if self.is_input_complete(&buffer) {
                        self.eval_input(&buffer);
                        buffer.clear();
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("{}",
                        "^C (Ctrl+C to interrupt, :quit to exit)".bright_yellow()
                    );
                    buffer.clear();
                }
                Err(ReadlineError::Eof) => {
                    println!("{}", "\nGoodbye!".bright_cyan());
                    break;
                }
                Err(err) => {
                    eprintln!("{} {}", "Error:".bright_red(), err);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handles special REPL commands starting with ':'
    /// Returns true to continue REPL, false to quit
    fn handle_command(&mut self, cmd: &str) -> bool {
        match cmd {
            ":help" | ":h" => {
                self.show_help();
                true
            }
            ":quit" | ":q" | ":exit" => {
                println!("{}", "Goodbye!".bright_cyan());
                false
            }
            ":clear" | ":c" => {
                // Clear the screen
                print!("\x1B[2J\x1B[1;1H");
                self.show_banner();
                true
            }
            ":vars" | ":v" => {
                self.show_variables();
                true
            }
            ":reset" | ":r" => {
                self.interpreter = Interpreter::new();
                println!("{}", "✓ Environment reset".bright_green());
                true
            }
            _ => {
                println!(
                    "{} Unknown command: {}. Type {}{}{}",
                    "Error:".bright_red(),
                    cmd.bright_yellow(),
                    ":".bright_blue(),
                    "help".bright_yellow(),
                    " for available commands.".bright_blue()
                );
                true
            }
        }
    }

    /// Displays help information about available commands
    fn show_help(&self) {
        println!();
        println!("{}", "REPL Commands:".bright_cyan().bold());
        println!();
        println!("  {}{}  Display this help message", ":help".bright_yellow(), " or :h     ".dimmed());
        println!("  {}{}  Exit the REPL", ":quit".bright_yellow(), " or :q     ".dimmed());
        println!("  {}{}  Clear the screen", ":clear".bright_yellow(), " or :c    ".dimmed());
        println!("  {}{}  Show defined variables", ":vars".bright_yellow(), " or :v    ".dimmed());
        println!("  {}{}  Reset environment", ":reset".bright_yellow(), " or :r   ".dimmed());
        println!();
        println!("{}", "Navigation:".bright_cyan().bold());
        println!();
        println!("  {}  Navigate command history", "↑/↓ arrows".bright_blue());
        println!("  {}  Interrupt current input", "Ctrl+C    ".bright_blue());
        println!("  {}  Exit REPL", "Ctrl+D    ".bright_blue());
        println!();
        println!("{}", "Multi-line Input:".bright_cyan().bold());
        println!();
        println!("  Leave braces, brackets, or parentheses unclosed to continue");
        println!("  on the next line. Close them to execute the statement.");
        println!();
        println!("{}", "Examples:".bright_cyan().bold());
        println!();
        println!("  {}",
            "ruff> let x := 42".dimmed()
        );
        println!("  {}",
            "ruff> func greet(name) {".dimmed()
        );
        println!("  {}",
            "....>     print(\"Hello, \" + name)".dimmed()
        );
        println!("  {}",
            "....> }".dimmed()
        );
        println!("  {}",
            "ruff> greet(\"World\")".dimmed()
        );
        println!();
    }

    /// Displays all currently defined variables in the environment
    fn show_variables(&self) {
        println!();
        println!("{}", "Defined Variables:".bright_cyan().bold());
        println!();

        // Get all variables from all scopes
        let _all_vars: HashMap<String, &Value> = HashMap::new();
        
        // Access the environment - we need to iterate through scopes
        // For now, we'll show this as a simplified view
        // In a full implementation, we'd expose more of the environment structure
        
        println!("  {}", "(Variable inspection not yet fully implemented)".dimmed());
        println!("  {}", "Tip: You can still use variables normally in the REPL".dimmed());
        println!();
    }

    /// Checks if the input is syntactically complete
    /// Returns true if all brackets/braces/parentheses are balanced
    fn is_input_complete(&self, input: &str) -> bool {
        let trimmed = input.trim();
        
        // Empty input is complete
        if trimmed.is_empty() {
            return true;
        }

        // Count unclosed delimiters
        let mut brace_count = 0;
        let mut bracket_count = 0;
        let mut paren_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut in_comment = false;

        for ch in trimmed.chars() {
            if in_comment {
                if ch == '\n' {
                    in_comment = false;
                }
                continue;
            }

            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => {
                    escape_next = true;
                }
                '"' => {
                    in_string = !in_string;
                }
                '#' if !in_string => {
                    in_comment = true;
                }
                '{' if !in_string => brace_count += 1,
                '}' if !in_string => brace_count -= 1,
                '[' if !in_string => bracket_count += 1,
                ']' if !in_string => bracket_count -= 1,
                '(' if !in_string => paren_count += 1,
                ')' if !in_string => paren_count -= 1,
                _ => {}
            }
        }

        // Input is complete if all delimiters are balanced and we're not in a string
        !in_string && brace_count == 0 && bracket_count == 0 && paren_count == 0
    }

    /// Evaluates the input code and displays the result
    fn eval_input(&mut self, input: &str) {
        let trimmed = input.trim();
        
        // Skip empty input
        if trimmed.is_empty() {
            return;
        }

        // Tokenize and parse
        let tokens = lexer::tokenize(input);
        let mut parser = parser::Parser::new(tokens);
        
        // Try to parse as expression first (for REPL convenience)
        // If that fails, try as statement
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            parser.parse()
        }));

        match result {
            Ok(stmts) if !stmts.is_empty() => {
                // Execute statements
                for stmt in &stmts {
                    match stmt {
                        // For expression statements in REPL, show the value
                        Stmt::ExprStmt(expr) => {
                            match self.interpreter.eval_expr_repl(expr) {
                                Ok(value) => {
                                    self.print_value(&value);
                                }
                                Err(err) => {
                                    self.print_error(&err);
                                }
                            }
                        }
                        // For other statements, just execute
                        _ => {
                            if let Err(err) = self.interpreter.eval_stmt_repl(stmt) {
                                self.print_error(&err);
                            }
                        }
                    }
                }
            }
            Ok(_) => {
                // Empty parse result
            }
            Err(_) => {
                println!("{} Failed to parse input", "Error:".bright_red());
            }
        }
    }

    /// Formats and displays a value
    fn print_value(&self, value: &Value) {
        match value {
            Value::Number(n) => {
                // Format number nicely
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    println!("{} {}", "=>".bright_blue(), (*n as i64).to_string().bright_white());
                } else {
                    println!("{} {}", "=>".bright_blue(), n.to_string().bright_white());
                }
            }
            Value::Str(s) => {
                println!("{} {}", "=>".bright_blue(), format!("\"{}\"", s).bright_green());
            }
            Value::Bool(b) => {
                println!("{} {}", "=>".bright_blue(), b.to_string().bright_magenta());
            }
            Value::Array(elements) => {
                print!("{} {}", "=>".bright_blue(), "[".bright_white());
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    print!("{}", self.format_value_inline(elem));
                }
                println!("{}", "]".bright_white());
            }
            Value::Dict(map) => {
                print!("{} {}", "=>".bright_blue(), "{".bright_white());
                for (i, (key, val)) in map.iter().enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    print!("{}: {}", 
                        format!("\"{}\"", key).bright_yellow(),
                        self.format_value_inline(val)
                    );
                }
                println!("{}", "}".bright_white());
            }
            Value::Function(params, _) => {
                println!(
                    "{} {}",
                    "=>".bright_blue(),
                    format!("<function({})>", params.join(", ")).bright_cyan()
                );
            }
            Value::Struct { name, fields } => {
                print!("{} {}", "=>".bright_blue(), format!("{} {{ ", name).bright_cyan());
                for (i, (key, val)) in fields.iter().enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    print!("{}: {}", key.bright_yellow(), self.format_value_inline(val));
                }
                println!("{}", " }".bright_cyan());
            }
            _ => {
                // For other types, use debug format
                println!("{} {:?}", "=>".bright_blue(), value);
            }
        }
    }

    /// Formats a value for inline display (used in arrays/dicts)
    fn format_value_inline(&self, value: &Value) -> String {
        match value {
            Value::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    (*n as i64).to_string()
                } else {
                    n.to_string()
                }
            }
            Value::Str(s) => format!("\"{}\"", s),
            Value::Bool(b) => b.to_string(),
            Value::Array(_) => "[...]".to_string(),
            Value::Dict(_) => "{...}".to_string(),
            Value::Function(params, _) => format!("<fn({})>", params.join(", ")),
            Value::Struct { name, .. } => format!("<{}>", name),
            _ => format!("{:?}", value),
        }
    }

    /// Displays an error message
    fn print_error(&self, err: &crate::errors::RuffError) {
        println!("{} {}", "Error:".bright_red().bold(), err.to_string().bright_red());
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new().expect("Failed to create REPL")
    }
}
