use crate::errors::{ErrorKind, RuffError};
use crate::interpreter::Value;
use crate::lexer::tokenize;
use crate::parser::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a loaded module with its exported symbols
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Module {
    pub name: String,
    pub path: PathBuf,
    pub exports: HashMap<String, Value>,
}

/// Manages module loading, caching, and resolution
pub struct ModuleLoader {
    /// Cache of loaded modules to avoid re-parsing
    loaded_modules: HashMap<String, Module>,
    /// Stack of modules currently being loaded (for circular import detection)
    loading_stack: Vec<String>,
    /// Search paths for module resolution
    search_paths: Vec<PathBuf>,
}

impl ModuleLoader {
    /// Creates a new module loader with default search paths
    pub fn new() -> Self {
        ModuleLoader {
            loaded_modules: HashMap::new(),
            loading_stack: Vec::new(),
            search_paths: vec![
                PathBuf::from("."),         // Current directory
                PathBuf::from("./modules"), // Local modules directory
            ],
        }
    }

    /// Adds a search path for module resolution
    #[allow(dead_code)]
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }

    /// Resolves a module name to a file path
    fn resolve_module_path(&self, module_name: &str) -> Option<PathBuf> {
        // Convert module name to file path (e.g., "math" -> "math.ruff")
        let filename = format!("{}.ruff", module_name);

        // Search in all search paths
        for search_path in &self.search_paths {
            let full_path = search_path.join(&filename);
            if full_path.exists() {
                return Some(full_path);
            }
        }

        None
    }

    /// Loads a module by name, returning cached version if available
    pub fn load_module(&mut self, module_name: &str) -> Result<Module, RuffError> {
        // Check cache first
        if let Some(module) = self.loaded_modules.get(module_name) {
            return Ok(module.clone());
        }

        // Check for circular imports
        if self.loading_stack.contains(&module_name.to_string()) {
            return Err(RuffError::new(
                ErrorKind::RuntimeError,
                format!("Circular import detected: {}", module_name),
                crate::errors::SourceLocation::unknown(),
            ));
        }

        // Resolve module path
        let module_path = self.resolve_module_path(module_name).ok_or_else(|| {
            RuffError::new(
                ErrorKind::RuntimeError,
                format!("Module not found: {}", module_name),
                crate::errors::SourceLocation::unknown(),
            )
        })?;

        // Add to loading stack
        self.loading_stack.push(module_name.to_string());

        // Read and parse module
        let source = fs::read_to_string(&module_path).map_err(|e| {
            RuffError::new(
                ErrorKind::RuntimeError,
                format!("Failed to read module {}: {}", module_name, e),
                crate::errors::SourceLocation::unknown(),
            )
        })?;

        let tokens = tokenize(&source);
        let mut parser = Parser::new(tokens);
        let _ast = parser.parse();

        // TODO: Execute module and collect exports
        // For now, create an empty module
        let module =
            Module { name: module_name.to_string(), path: module_path, exports: HashMap::new() };

        // Remove from loading stack
        self.loading_stack.pop();

        // Cache the module
        self.loaded_modules.insert(module_name.to_string(), module.clone());

        Ok(module)
    }

    /// Gets a specific symbol from a module
    pub fn get_symbol(&mut self, module_name: &str, symbol_name: &str) -> Result<Value, RuffError> {
        let module = self.load_module(module_name)?;

        module.exports.get(symbol_name).cloned().ok_or_else(|| {
            RuffError::new(
                ErrorKind::RuntimeError,
                format!("Symbol '{}' not found in module '{}'", symbol_name, module_name),
                crate::errors::SourceLocation::unknown(),
            )
        })
    }

    /// Gets all exports from a module
    pub fn get_all_exports(
        &mut self,
        module_name: &str,
    ) -> Result<HashMap<String, Value>, RuffError> {
        let module = self.load_module(module_name)?;
        Ok(module.exports.clone())
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}
