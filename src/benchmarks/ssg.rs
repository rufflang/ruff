use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SsgStageProfile {
    pub read_ms: f64,
    pub render_write_ms: f64,
}

impl SsgStageProfile {
    pub fn total_profiled_ms(&self) -> f64 {
        self.read_ms + self.render_write_ms
    }

    pub fn bottleneck_stage(&self) -> Option<(&'static str, f64, f64)> {
        let total = self.total_profiled_ms();
        if total <= 0.0 {
            return None;
        }

        if self.read_ms >= self.render_write_ms {
            Some(("read", self.read_ms, (self.read_ms / total) * 100.0))
        } else {
            Some(("render/write", self.render_write_ms, (self.render_write_ms / total) * 100.0))
        }
    }
}

#[derive(Debug, Clone)]
pub struct SsgBenchmarkResult {
    pub files: usize,
    pub ruff_build_ms: f64,
    pub ruff_files_per_sec: f64,
    pub ruff_checksum: i128,
    pub ruff_stage_profile: Option<SsgStageProfile>,
    pub python_build_ms: Option<f64>,
    pub python_files_per_sec: Option<f64>,
    pub python_stage_profile: Option<SsgStageProfile>,
}

impl SsgBenchmarkResult {
    pub fn ruff_vs_python_speedup(&self) -> Option<f64> {
        self.python_build_ms.map(|python_ms| {
            if self.ruff_build_ms <= 0.0 {
                0.0
            } else {
                python_ms / self.ruff_build_ms
            }
        })
    }
}

fn parse_metric_value(output: &str, metric_key: &str) -> Result<f64, String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some((key, value)) = trimmed.split_once('=') {
            if key.trim() == metric_key {
                let parsed = value.trim().parse::<f64>().map_err(|e| {
                    format!("Metric '{}' had invalid numeric value '{}': {}", metric_key, value, e)
                })?;
                return Ok(parsed);
            }
        }
    }

    Err(format!("Metric '{}' not found in output", metric_key))
}

fn parse_metric_value_optional(output: &str, metric_key: &str) -> Result<Option<f64>, String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some((key, value)) = trimmed.split_once('=') {
            if key.trim() == metric_key {
                let parsed = value.trim().parse::<f64>().map_err(|e| {
                    format!("Metric '{}' had invalid numeric value '{}': {}", metric_key, value, e)
                })?;
                return Ok(Some(parsed));
            }
        }
    }

    Ok(None)
}

fn parse_metric_usize(output: &str, metric_key: &str) -> Result<usize, String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some((key, value)) = trimmed.split_once('=') {
            if key.trim() == metric_key {
                let parsed = value.trim().parse::<usize>().map_err(|e| {
                    format!("Metric '{}' had invalid integer value '{}': {}", metric_key, value, e)
                })?;
                return Ok(parsed);
            }
        }
    }

    Err(format!("Metric '{}' not found in output", metric_key))
}

fn parse_checksum(output: &str, metric_key: &str) -> Result<i128, String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some((key, value)) = trimmed.split_once('=') {
            if key.trim() == metric_key {
                let parsed = value.trim().parse::<i128>().map_err(|e| {
                    format!(
                        "Checksum '{}' had invalid integer value '{}': {}",
                        metric_key, value, e
                    )
                })?;
                return Ok(parsed);
            }
        }
    }

    Err(format!("Checksum '{}' not found in output", metric_key))
}

fn run_and_capture(program: &str, args: &[&str], working_dir: &Path) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(working_dir)
        .output()
        .map_err(|e| format!("Failed to run '{}': {}", program, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined =
        if stderr.trim().is_empty() { stdout } else { format!("{}\n{}", stdout, stderr) };

    if !output.status.success() {
        return Err(format!(
            "Command '{}' failed with status {:?}:\n{}",
            program,
            output.status.code(),
            combined
        ));
    }

    Ok(combined)
}

fn determine_workspace_root(script_path: &Path) -> PathBuf {
    let absolute_script_path = if script_path.is_absolute() {
        script_path.to_path_buf()
    } else {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        current_dir.join(script_path)
    };

    let mut current = absolute_script_path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();

    loop {
        if current.join("Cargo.toml").exists() || current.join("tmp").exists() {
            return current;
        }

        if !current.pop() {
            break;
        }
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn run_ssg_benchmark(
    ruff_binary: &Path,
    ruff_script: &Path,
    python_binary: Option<&str>,
    python_script: Option<&Path>,
) -> Result<SsgBenchmarkResult, String> {
    let ruff_binary_str = ruff_binary
        .to_str()
        .ok_or_else(|| format!("Invalid Ruff binary path: {}", ruff_binary.display()))?;
    let ruff_script_str = ruff_script
        .to_str()
        .ok_or_else(|| format!("Invalid Ruff script path: {}", ruff_script.display()))?;
    let working_dir = determine_workspace_root(ruff_script);

    let ruff_output = run_and_capture(ruff_binary_str, &["run", ruff_script_str], &working_dir)?;

    let files = parse_metric_usize(&ruff_output, "RUFF_SSG_FILES")?;
    let ruff_build_ms = parse_metric_value(&ruff_output, "RUFF_SSG_BUILD_MS")?;
    let ruff_files_per_sec = parse_metric_value(&ruff_output, "RUFF_SSG_FILES_PER_SEC")?;
    let ruff_checksum = parse_checksum(&ruff_output, "RUFF_SSG_CHECKSUM")?;
    let ruff_read_ms = parse_metric_value_optional(&ruff_output, "RUFF_SSG_READ_MS")?;
    let ruff_render_write_ms =
        parse_metric_value_optional(&ruff_output, "RUFF_SSG_RENDER_WRITE_MS")?;

    let ruff_stage_profile = match (ruff_read_ms, ruff_render_write_ms) {
        (Some(read_ms), Some(render_write_ms)) => {
            Some(SsgStageProfile { read_ms, render_write_ms })
        }
        _ => None,
    };

    let mut result = SsgBenchmarkResult {
        files,
        ruff_build_ms,
        ruff_files_per_sec,
        ruff_checksum,
        ruff_stage_profile,
        python_build_ms: None,
        python_files_per_sec: None,
        python_stage_profile: None,
    };

    if let (Some(python_binary), Some(python_script)) = (python_binary, python_script) {
        let python_script_str = python_script
            .to_str()
            .ok_or_else(|| format!("Invalid Python script path: {}", python_script.display()))?;

        let python_output = run_and_capture(python_binary, &[python_script_str], &working_dir)?;
        let python_files = parse_metric_usize(&python_output, "PYTHON_SSG_FILES")?;
        let python_build_ms = parse_metric_value(&python_output, "PYTHON_SSG_BUILD_MS")?;
        let python_files_per_sec = parse_metric_value(&python_output, "PYTHON_SSG_FILES_PER_SEC")?;
        let python_checksum = parse_checksum(&python_output, "PYTHON_SSG_CHECKSUM")?;
        let python_read_ms = parse_metric_value_optional(&python_output, "PYTHON_SSG_READ_MS")?;
        let python_render_write_ms =
            parse_metric_value_optional(&python_output, "PYTHON_SSG_RENDER_WRITE_MS")?;

        let python_stage_profile = match (python_read_ms, python_render_write_ms) {
            (Some(read_ms), Some(render_write_ms)) => {
                Some(SsgStageProfile { read_ms, render_write_ms })
            }
            _ => None,
        };

        if python_files != result.files {
            return Err(format!(
                "File count mismatch: Ruff={} Python={} (benchmarks must use identical workload)",
                result.files, python_files
            ));
        }

        if python_checksum != result.ruff_checksum {
            return Err(format!(
                "Checksum mismatch: Ruff={} Python={} (outputs are not equivalent)",
                result.ruff_checksum, python_checksum
            ));
        }

        result.python_build_ms = Some(python_build_ms);
        result.python_files_per_sec = Some(python_files_per_sec);
        result.python_stage_profile = python_stage_profile;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metric_value_extracts_float() {
        let output = "noise\nRUFF_SSG_BUILD_MS=12.500\nother";
        let value = parse_metric_value(output, "RUFF_SSG_BUILD_MS").unwrap();
        assert!((value - 12.5).abs() < 0.0001);
    }

    #[test]
    fn test_parse_metric_usize_extracts_integer() {
        let output = "RUFF_SSG_FILES=10000";
        let value = parse_metric_usize(output, "RUFF_SSG_FILES").unwrap();
        assert_eq!(value, 10000);
    }

    #[test]
    fn test_parse_checksum_extracts_integer() {
        let output = "PYTHON_SSG_CHECKSUM=424242";
        let value = parse_checksum(output, "PYTHON_SSG_CHECKSUM").unwrap();
        assert_eq!(value, 424242);
    }

    #[test]
    fn test_parse_checksum_invalid_value_returns_error() {
        let output = "RUFF_SSG_CHECKSUM=not_a_number";
        let err = parse_checksum(output, "RUFF_SSG_CHECKSUM").unwrap_err();
        assert!(err.contains("invalid integer value"));
    }

    #[test]
    fn test_parse_metric_value_missing_key_returns_error() {
        let output = "RUFF_SSG_BUILD_MS=1.0";
        let err = parse_metric_value(output, "PYTHON_SSG_BUILD_MS").unwrap_err();
        assert!(err.contains("Metric 'PYTHON_SSG_BUILD_MS' not found"));
    }

    #[test]
    fn test_parse_metric_usize_invalid_value_returns_error() {
        let output = "RUFF_SSG_FILES=notanumber";
        let err = parse_metric_usize(output, "RUFF_SSG_FILES").unwrap_err();
        assert!(err.contains("invalid integer value"));
    }

    #[test]
    fn test_parse_metric_value_optional_present() {
        let output = "RUFF_SSG_READ_MS=14.25";
        let value = parse_metric_value_optional(output, "RUFF_SSG_READ_MS").unwrap();
        assert_eq!(value, Some(14.25));
    }

    #[test]
    fn test_parse_metric_value_optional_absent_returns_none() {
        let output = "RUFF_SSG_BUILD_MS=120.0";
        let value = parse_metric_value_optional(output, "RUFF_SSG_RENDER_WRITE_MS").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_parse_metric_value_optional_invalid_returns_error() {
        let output = "RUFF_SSG_RENDER_WRITE_MS=fast";
        let err = parse_metric_value_optional(output, "RUFF_SSG_RENDER_WRITE_MS").unwrap_err();
        assert!(err.contains("invalid numeric value"));
    }

    #[test]
    fn test_stage_profile_total_profiled_ms() {
        let profile = SsgStageProfile { read_ms: 25.0, render_write_ms: 75.0 };
        assert_eq!(profile.total_profiled_ms(), 100.0);
    }

    #[test]
    fn test_stage_profile_bottleneck_read() {
        let profile = SsgStageProfile { read_ms: 80.0, render_write_ms: 20.0 };
        let bottleneck = profile.bottleneck_stage().unwrap();
        assert_eq!(bottleneck.0, "read");
        assert!((bottleneck.1 - 80.0).abs() < 0.0001);
        assert!((bottleneck.2 - 80.0).abs() < 0.0001);
    }

    #[test]
    fn test_stage_profile_bottleneck_render_write() {
        let profile = SsgStageProfile { read_ms: 30.0, render_write_ms: 70.0 };
        let bottleneck = profile.bottleneck_stage().unwrap();
        assert_eq!(bottleneck.0, "render/write");
        assert!((bottleneck.1 - 70.0).abs() < 0.0001);
        assert!((bottleneck.2 - 70.0).abs() < 0.0001);
    }

    #[test]
    fn test_stage_profile_bottleneck_zero_total_returns_none() {
        let profile = SsgStageProfile { read_ms: 0.0, render_write_ms: 0.0 };
        assert!(profile.bottleneck_stage().is_none());
    }

    #[test]
    fn test_speedup_calculation() {
        let result = SsgBenchmarkResult {
            files: 10000,
            ruff_build_ms: 1000.0,
            ruff_files_per_sec: 10000.0,
            ruff_checksum: 100,
            ruff_stage_profile: None,
            python_build_ms: Some(2500.0),
            python_files_per_sec: Some(4000.0),
            python_stage_profile: None,
        };

        assert!((result.ruff_vs_python_speedup().unwrap() - 2.5).abs() < 0.0001);
    }

    #[test]
    fn test_speedup_calculation_without_python_result() {
        let result = SsgBenchmarkResult {
            files: 10000,
            ruff_build_ms: 1000.0,
            ruff_files_per_sec: 10000.0,
            ruff_checksum: 100,
            ruff_stage_profile: None,
            python_build_ms: None,
            python_files_per_sec: None,
            python_stage_profile: None,
        };

        assert!(result.ruff_vs_python_speedup().is_none());
    }

    #[test]
    fn test_speedup_calculation_handles_zero_ruff_duration() {
        let result = SsgBenchmarkResult {
            files: 10000,
            ruff_build_ms: 0.0,
            ruff_files_per_sec: 0.0,
            ruff_checksum: 100,
            ruff_stage_profile: None,
            python_build_ms: Some(1000.0),
            python_files_per_sec: Some(10000.0),
            python_stage_profile: None,
        };

        assert_eq!(result.ruff_vs_python_speedup().unwrap(), 0.0);
    }
}
