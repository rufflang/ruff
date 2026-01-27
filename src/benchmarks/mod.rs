// Benchmarking module for Ruff performance testing
//
// This module provides infrastructure for benchmarking Ruff execution modes:
// - Tree-walking interpreter
// - Bytecode VM
// - JIT-compiled native code
//
// Usage:
//   let bench = BenchmarkRunner::new();
//   bench.run_all();

pub mod timer;
pub mod stats;
pub mod runner;
pub mod reporter;

pub use runner::BenchmarkRunner;
pub use timer::Timer;
pub use stats::Statistics;
pub use reporter::Reporter;

#[derive(Debug, Clone)]
pub enum ExecutionMode {
    Interpreter,
    VM,
    JIT,
}

impl ExecutionMode {
    pub fn name(&self) -> &str {
        match self {
            ExecutionMode::Interpreter => "Interpreter",
            ExecutionMode::VM => "VM",
            ExecutionMode::JIT => "JIT",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub mode: ExecutionMode,
    pub samples: Vec<std::time::Duration>,
    pub success: bool,
    pub error: Option<String>,
}

impl BenchmarkResult {
    pub fn new(name: String, mode: ExecutionMode) -> Self {
        Self {
            name,
            mode,
            samples: Vec::new(),
            success: true,
            error: None,
        }
    }

    pub fn add_sample(&mut self, duration: std::time::Duration) {
        self.samples.push(duration);
    }

    pub fn set_error(&mut self, error: String) {
        self.success = false;
        self.error = Some(error);
    }

    pub fn mean(&self) -> Option<std::time::Duration> {
        if self.samples.is_empty() {
            return None;
        }
        let total: std::time::Duration = self.samples.iter().sum();
        Some(total / self.samples.len() as u32)
    }

    pub fn median(&self) -> Option<std::time::Duration> {
        if self.samples.is_empty() {
            return None;
        }
        let mut sorted = self.samples.clone();
        sorted.sort();
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            Some((sorted[mid - 1] + sorted[mid]) / 2)
        } else {
            Some(sorted[mid])
        }
    }

    pub fn min(&self) -> Option<std::time::Duration> {
        self.samples.iter().min().copied()
    }

    pub fn max(&self) -> Option<std::time::Duration> {
        self.samples.iter().max().copied()
    }
}
