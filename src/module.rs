use crate::errors::{ErrorKind, RuffError};
use crate::ast::{Expr, Pattern, Stmt};
use crate::interpreter::{Environment, Interpreter, Value};
use crate::lexer::tokenize;
use crate::parser::Parser;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
    fn runtime_error(message: String) -> Box<RuffError> {
        Box::new(RuffError::new(
            ErrorKind::RuntimeError,
            message,
            crate::errors::SourceLocation::unknown(),
        ))
    }

    fn collect_pattern_bindings(pattern: &Pattern, names: &mut Vec<String>) {
        match pattern {
            Pattern::Identifier(name) => names.push(name.clone()),
            Pattern::Array { elements, rest } => {
                for element in elements {
                    Self::collect_pattern_bindings(element, names);
                }
                if let Some(rest_name) = rest {
                    names.push(rest_name.clone());
                }
            }
            Pattern::Dict { keys, rest } => {
                names.extend(keys.iter().cloned());
                if let Some(rest_name) = rest {
                    names.push(rest_name.clone());
                }
            }
            Pattern::Ignore => {}
        }
    }

    fn collect_export_bindings(stmt: &Stmt, names: &mut Vec<String>) {
        match stmt {
            Stmt::Let { pattern, .. } => Self::collect_pattern_bindings(pattern, names),
            Stmt::Const { name, .. }
            | Stmt::FuncDef { name, .. }
            | Stmt::StructDef { name, .. } => names.push(name.clone()),
            Stmt::EnumDef { name, variants } => {
                for variant in variants {
                    names.push(format!("{}::{}", name, variant));
                }
            }
            Stmt::Assign { target, .. } => {
                if let Expr::Identifier(name) = target {
                    names.push(name.clone());
                }
            }
            Stmt::ExprStmt(Expr::Identifier(name)) => names.push(name.clone()),
            Stmt::Block(stmts) => {
                for nested_stmt in stmts {
                    Self::collect_export_bindings(nested_stmt, names);
                }
            }
            Stmt::Export { stmt } => Self::collect_export_bindings(stmt, names),
            _ => {}
        }
    }

    fn collect_exported_symbol_names(program: &[Stmt]) -> Vec<String> {
        let mut names = Vec::new();
        for stmt in program {
            if let Stmt::Export { stmt } = stmt {
                Self::collect_export_bindings(stmt, &mut names);
            }
        }

        let mut deduped = Vec::new();
        let mut seen = HashSet::new();
        for name in names {
            if seen.insert(name.clone()) {
                deduped.push(name);
            }
        }

        deduped
    }

    fn bind_export_value_to_module_env(
        value: Value,
        module_env: &Arc<Mutex<Environment>>,
    ) -> Value {
        match value {
            Value::Function(params, body, captured_env) => {
                let bound_env = captured_env.or_else(|| Some(Arc::clone(module_env)));
                Value::Function(params, body, bound_env)
            }
            Value::AsyncFunction(params, body, captured_env) => {
                let bound_env = captured_env.or_else(|| Some(Arc::clone(module_env)));
                Value::AsyncFunction(params, body, bound_env)
            }
            Value::StructDef { name, field_names, methods } => {
                let mut bound_methods = HashMap::new();
                for (method_name, method_value) in methods {
                    bound_methods.insert(
                        method_name,
                        Self::bind_export_value_to_module_env(method_value, module_env),
                    );
                }
                Value::StructDef { name, field_names, methods: bound_methods }
            }
            other => other,
        }
    }

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
    pub fn load_module(&mut self, module_name: &str) -> Result<Module, Box<RuffError>> {
        // Check cache first
        if let Some(module) = self.loaded_modules.get(module_name) {
            return Ok(module.clone());
        }

        // Check for circular imports
        if self.loading_stack.iter().any(|name| name == module_name) {
            return Err(Self::runtime_error(format!("Circular import detected: {}", module_name)));
        }

        // Resolve module path
        let module_path = self
            .resolve_module_path(module_name)
            .ok_or_else(|| Self::runtime_error(format!("Module not found: {}", module_name)))?;

        // Add to loading stack
        self.loading_stack.push(module_name.to_string());

        let load_result = (|| {
            let source = fs::read_to_string(&module_path).map_err(|e| {
                Self::runtime_error(format!("Failed to read module {}: {}", module_name, e))
            })?;

            let tokens = tokenize(&source);
            let mut parser = Parser::new(tokens);
            let program = parser.parse();
            let export_names = Self::collect_exported_symbol_names(&program);

            let mut interpreter = Interpreter::new();
            let mut active_loader = std::mem::take(self);
            if let Some(parent) = module_path.parent() {
                active_loader.add_search_path(parent);
            }
            interpreter.module_loader = active_loader;
            interpreter.source_file = Some(module_path.to_string_lossy().to_string());
            interpreter.eval_stmts(&program);

            *self = std::mem::take(&mut interpreter.module_loader);

            if let Some(return_value) = interpreter.return_value.take() {
                match return_value {
                    Value::Error(message) => {
                        return Err(Self::runtime_error(format!(
                            "Failed to evaluate module '{}': {}",
                            module_name, message
                        )));
                    }
                    Value::ErrorObject { message, .. } => {
                        return Err(Self::runtime_error(format!(
                            "Failed to evaluate module '{}': {}",
                            module_name, message
                        )));
                    }
                    _ => {}
                }
            }

            let module_env = Arc::new(Mutex::new(interpreter.env.clone()));

            let mut exports = HashMap::new();
            for export_name in export_names {
                if let Some(value) = interpreter.env.get(&export_name) {
                    let bound_value = Self::bind_export_value_to_module_env(value, &module_env);
                    exports.insert(export_name, bound_value);
                } else {
                    return Err(Self::runtime_error(format!(
                        "Exported symbol '{}' was not defined in module '{}'",
                        export_name, module_name
                    )));
                }
            }

            Ok(Module { name: module_name.to_string(), path: module_path.clone(), exports })
        })();

        self.loading_stack.pop();

        let module = load_result?;
        self.loaded_modules.insert(module_name.to_string(), module.clone());

        Ok(module)
    }

    /// Gets a specific symbol from a module
    pub fn get_symbol(
        &mut self,
        module_name: &str,
        symbol_name: &str,
    ) -> Result<Value, Box<RuffError>> {
        let module = self.load_module(module_name)?;

        module.exports.get(symbol_name).cloned().ok_or_else(|| {
            Box::new(RuffError::new(
                ErrorKind::RuntimeError,
                format!("Symbol '{}' not found in module '{}'", symbol_name, module_name),
                crate::errors::SourceLocation::unknown(),
            ))
        })
    }

    /// Gets all exports from a module
    pub fn get_all_exports(
        &mut self,
        module_name: &str,
    ) -> Result<HashMap<String, Value>, Box<RuffError>> {
        let module = self.load_module(module_name)?;
        Ok(module.exports.clone())
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_name(prefix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        format!("{}_{}_{}", prefix, std::process::id(), nanos)
    }

    #[test]
    fn load_module_collects_explicit_exports_only() {
        let mut loader = ModuleLoader::new();
        let temp_root = std::env::temp_dir().join(unique_name("ruff_module_exports"));
        fs::create_dir_all(&temp_root).expect("failed to create temp module dir");

        let module_name = unique_name("math_mod");
        let module_path = temp_root.join(format!("{}.ruff", module_name));
        fs::write(
            &module_path,
            "helper := 10\nexport visible := helper + 5\nhidden := 99\n",
        )
        .expect("failed to write module source");

        loader.add_search_path(&temp_root);

        let exports = loader
            .get_all_exports(&module_name)
            .expect("module should load and return exports");

        assert!(matches!(exports.get("visible"), Some(Value::Int(15))));
        assert!(exports.get("helper").is_none());
        assert!(exports.get("hidden").is_none());

        fs::remove_file(&module_path).expect("failed to clean up module file");
        fs::remove_dir_all(&temp_root).expect("failed to clean up temp module dir");
    }

    #[test]
    fn get_symbol_reports_missing_symbol_deterministically() {
        let mut loader = ModuleLoader::new();
        let temp_root = std::env::temp_dir().join(unique_name("ruff_module_missing_symbol"));
        fs::create_dir_all(&temp_root).expect("failed to create temp module dir");

        let module_name = unique_name("missing_symbol_mod");
        let module_path = temp_root.join(format!("{}.ruff", module_name));
        fs::write(&module_path, "export present := 1\n").expect("failed to write module source");

        loader.add_search_path(&temp_root);

        let err = loader
            .get_symbol(&module_name, "absent")
            .expect_err("expected missing symbol error");

        assert!(err.message.contains(&format!(
            "Symbol 'absent' not found in module '{}'",
            module_name
        )));

        fs::remove_file(&module_path).expect("failed to clean up module file");
        fs::remove_dir_all(&temp_root).expect("failed to clean up temp module dir");
    }

    #[test]
    fn load_module_detects_circular_imports() {
        let mut loader = ModuleLoader::new();
        let temp_root = std::env::temp_dir().join(unique_name("ruff_module_circular"));
        fs::create_dir_all(&temp_root).expect("failed to create temp module dir");

        let module_a = unique_name("module_a");
        let module_b = unique_name("module_b");
        let module_a_path = temp_root.join(format!("{}.ruff", module_a));
        let module_b_path = temp_root.join(format!("{}.ruff", module_b));

        fs::write(&module_a_path, format!("import {}\nexport a := 1\n", module_b))
            .expect("failed to write module A");
        fs::write(&module_b_path, format!("import {}\nexport b := 2\n", module_a))
            .expect("failed to write module B");

        loader.add_search_path(&temp_root);

        let err = loader
            .get_all_exports(&module_a)
            .expect_err("expected circular import failure");

        assert!(err.message.contains("Circular import detected"));

        fs::remove_file(&module_a_path).expect("failed to clean up module A");
        fs::remove_file(&module_b_path).expect("failed to clean up module B");
        fs::remove_dir_all(&temp_root).expect("failed to clean up temp module dir");
    }
}
