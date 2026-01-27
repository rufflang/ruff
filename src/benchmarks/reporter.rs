// Results formatting and reporting

use crate::benchmarks::{BenchmarkResult, ExecutionMode, Statistics};
use colored::*;
use std::time::Duration;

pub struct Reporter;

impl Reporter {
    pub fn print_header(title: &str) {
        let width = 80;
        println!("{}", "=".repeat(width).bright_blue());
        println!("{:^width$}", title.bright_white().bold(), width = width);
        println!("{}", "=".repeat(width).bright_blue());
        println!();
    }

    pub fn print_separator() {
        println!("{}", "-".repeat(80).blue());
    }

    pub fn print_benchmark_result(result: &BenchmarkResult) {
        if !result.success {
            println!("{} {}", 
                "✗".red().bold(),
                format!("{} ({}) - {}", 
                    result.name, 
                    result.mode.name(),
                    result.error.as_ref().unwrap_or(&"Unknown error".to_string())
                ).red()
            );
            return;
        }

        if let Some(stats) = Statistics::from_samples(&result.samples) {
            println!("{} {} ({})", 
                "✓".green().bold(),
                result.name.bright_white(),
                result.mode.name().cyan()
            );
            println!("  Mean:   {}", Statistics::format_duration(stats.mean).yellow());
            println!("  Median: {}", Statistics::format_duration(stats.median).yellow());
            println!("  Min:    {}", Statistics::format_duration(stats.min).green());
            println!("  Max:    {}", Statistics::format_duration(stats.max).red());
            println!("  StdDev: {}", Statistics::format_duration(stats.stddev).blue());
            println!("  Samples: {}", stats.samples);
        }
    }

    pub fn print_comparison_table(results: &[(String, Vec<BenchmarkResult>)]) {
        println!();
        Self::print_header("Performance Comparison");

        // Table header
        println!(
            "{:<30} {:>15} {:>15} {:>15} {:>15}",
            "Benchmark".bright_white().bold(),
            "Interpreter".bright_white().bold(),
            "VM".bright_white().bold(),
            "JIT".bright_white().bold(),
            "Speedup".bright_white().bold()
        );
        Self::print_separator();

        for (name, bench_results) in results {
            let mut interp_time = None;
            let mut vm_time = None;
            let mut jit_time = None;

            for result in bench_results {
                if result.success {
                    let mean = result.mean();
                    match result.mode {
                        ExecutionMode::Interpreter => interp_time = mean,
                        ExecutionMode::VM => vm_time = mean,
                        ExecutionMode::JIT => jit_time = mean,
                    }
                }
            }

            let interp_str = if let Some(t) = interp_time {
                Statistics::format_duration(t)
            } else {
                "N/A".to_string()
            };

            let vm_str = if let Some(t) = vm_time {
                Statistics::format_duration(t)
            } else {
                "N/A".to_string()
            };

            let jit_str = if let Some(t) = jit_time {
                Statistics::format_duration(t)
            } else {
                "N/A".to_string()
            };

            let speedup_str = if let (Some(interp), Some(jit)) = (interp_time, jit_time) {
                let speedup = interp.as_nanos() as f64 / jit.as_nanos() as f64;
                format!("{:.2}x", speedup)
            } else {
                "N/A".to_string()
            };

            println!(
                "{:<30} {:>15} {:>15} {:>15} {:>15}",
                name.bright_white(),
                interp_str.yellow(),
                vm_str.cyan(),
                jit_str.green(),
                speedup_str.bright_green().bold()
            );
        }

        println!();
    }

    pub fn print_summary(results: &[(String, Vec<BenchmarkResult>)]) {
        let mut total_interp = Duration::ZERO;
        let mut total_vm = Duration::ZERO;
        let mut total_jit = Duration::ZERO;
        let mut count = 0;

        for (_name, bench_results) in results {
            for result in bench_results {
                if result.success {
                    if let Some(mean) = result.mean() {
                        match result.mode {
                            ExecutionMode::Interpreter => total_interp += mean,
                            ExecutionMode::VM => total_vm += mean,
                            ExecutionMode::JIT => total_jit += mean,
                        }
                    }
                }
            }
            count += 1;
        }

        if count > 0 {
            println!();
            Self::print_header("Summary");
            println!("Total Benchmarks: {}", count.to_string().bright_white().bold());
            println!("Total Time (Interpreter): {}", 
                Statistics::format_duration(total_interp).yellow());
            println!("Total Time (VM): {}", 
                Statistics::format_duration(total_vm).cyan());
            println!("Total Time (JIT): {}", 
                Statistics::format_duration(total_jit).green());

            if total_interp.as_nanos() > 0 {
                let vm_speedup = total_interp.as_nanos() as f64 / total_vm.as_nanos() as f64;
                let jit_speedup = total_interp.as_nanos() as f64 / total_jit.as_nanos() as f64;
                println!();
                println!("Average VM Speedup: {}", 
                    format!("{:.2}x", vm_speedup).bright_green().bold());
                println!("Average JIT Speedup: {}", 
                    format!("{:.2}x", jit_speedup).bright_green().bold());
            }
        }
    }
}
