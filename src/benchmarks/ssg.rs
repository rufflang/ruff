use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SsgRunStatistics {
    pub runs: usize,
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub stddev: f64,
}

impl SsgRunStatistics {
    pub fn from_samples(samples: &[f64]) -> Option<Self> {
        if samples.is_empty() {
            return None;
        }

        let runs = samples.len();
        let sum: f64 = samples.iter().sum();
        let mean = sum / runs as f64;

        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));

        let median = if runs % 2 == 0 {
            let right = runs / 2;
            let left = right - 1;
            (sorted[left] + sorted[right]) / 2.0
        } else {
            sorted[runs / 2]
        };

        let min = *sorted.first().unwrap_or(&0.0);
        let max = *sorted.last().unwrap_or(&0.0);

        let variance = samples
            .iter()
            .map(|value| {
                let diff = *value - mean;
                diff * diff
            })
            .sum::<f64>()
            / runs as f64;

        let stddev = variance.sqrt();

        Some(SsgRunStatistics { runs, mean, median, min, max, stddev })
    }
}

#[derive(Debug, Clone)]
pub struct SsgStageProfileStatistics {
    pub read_ms: SsgRunStatistics,
    pub render_write_ms: SsgRunStatistics,
}

#[derive(Debug, Clone)]
pub struct SsgBenchmarkAggregateResult {
    pub files: usize,
    pub ruff_checksum: i128,
    pub ruff_build_ms: SsgRunStatistics,
    pub ruff_files_per_sec: SsgRunStatistics,
    pub ruff_stage_profile: Option<SsgStageProfileStatistics>,
    pub python_build_ms: Option<SsgRunStatistics>,
    pub python_files_per_sec: Option<SsgRunStatistics>,
    pub python_stage_profile: Option<SsgStageProfileStatistics>,
    pub ruff_vs_python_speedup: Option<SsgRunStatistics>,
}

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

pub fn aggregate_ssg_results(
    run_results: &[SsgBenchmarkResult],
) -> Result<SsgBenchmarkAggregateResult, String> {
    if run_results.is_empty() {
        return Err("Cannot aggregate SSG benchmark results: no runs provided".to_string());
    }

    let baseline = &run_results[0];
    let files = baseline.files;
    let checksum = baseline.ruff_checksum;

    let mut ruff_build_samples = Vec::with_capacity(run_results.len());
    let mut ruff_throughput_samples = Vec::with_capacity(run_results.len());
    let mut ruff_read_samples = Vec::new();
    let mut ruff_render_write_samples = Vec::new();

    let mut python_build_samples = Vec::new();
    let mut python_throughput_samples = Vec::new();
    let mut python_read_samples = Vec::new();
    let mut python_render_write_samples = Vec::new();
    let mut speedup_samples = Vec::new();

    let all_have_python = run_results
        .iter()
        .all(|result| result.python_build_ms.is_some() && result.python_files_per_sec.is_some());
    let none_have_python = run_results
        .iter()
        .all(|result| result.python_build_ms.is_none() && result.python_files_per_sec.is_none());

    if !all_have_python && !none_have_python {
        return Err(
            "Cannot aggregate SSG benchmark results: inconsistent Python comparison presence across runs"
                .to_string(),
        );
    }

    for result in run_results {
        if result.files != files {
            return Err(format!(
                "Cannot aggregate SSG benchmark results: file count mismatch across runs (expected {}, got {})",
                files, result.files
            ));
        }

        if result.ruff_checksum != checksum {
            return Err(format!(
                "Cannot aggregate SSG benchmark results: Ruff checksum mismatch across runs (expected {}, got {})",
                checksum, result.ruff_checksum
            ));
        }

        ruff_build_samples.push(result.ruff_build_ms);
        ruff_throughput_samples.push(result.ruff_files_per_sec);

        if let Some(profile) = result.ruff_stage_profile.as_ref() {
            ruff_read_samples.push(profile.read_ms);
            ruff_render_write_samples.push(profile.render_write_ms);
        }

        if all_have_python {
            let python_build_ms = result.python_build_ms.ok_or_else(|| {
                "Cannot aggregate SSG benchmark results: missing Python build metric".to_string()
            })?;
            let python_files_per_sec = result.python_files_per_sec.ok_or_else(|| {
                "Cannot aggregate SSG benchmark results: missing Python throughput metric"
                    .to_string()
            })?;

            python_build_samples.push(python_build_ms);
            python_throughput_samples.push(python_files_per_sec);

            if let Some(profile) = result.python_stage_profile.as_ref() {
                python_read_samples.push(profile.read_ms);
                python_render_write_samples.push(profile.render_write_ms);
            }

            if let Some(speedup) = result.ruff_vs_python_speedup() {
                speedup_samples.push(speedup);
            } else {
                return Err(
                    "Cannot aggregate SSG benchmark results: missing speedup metric for Python comparison"
                        .to_string(),
                );
            }
        }
    }

    let ruff_stage_profile = if ruff_read_samples.len() == run_results.len()
        && ruff_render_write_samples.len() == run_results.len()
    {
        Some(SsgStageProfileStatistics {
            read_ms: SsgRunStatistics::from_samples(&ruff_read_samples)
                .ok_or_else(|| "Cannot aggregate Ruff stage profile read metrics".to_string())?,
            render_write_ms: SsgRunStatistics::from_samples(&ruff_render_write_samples)
                .ok_or_else(|| {
                    "Cannot aggregate Ruff stage profile render/write metrics".to_string()
                })?,
        })
    } else {
        None
    };

    let python_stage_profile = if all_have_python
        && python_read_samples.len() == run_results.len()
        && python_render_write_samples.len() == run_results.len()
    {
        Some(SsgStageProfileStatistics {
            read_ms: SsgRunStatistics::from_samples(&python_read_samples)
                .ok_or_else(|| "Cannot aggregate Python stage profile read metrics".to_string())?,
            render_write_ms: SsgRunStatistics::from_samples(&python_render_write_samples)
                .ok_or_else(|| {
                    "Cannot aggregate Python stage profile render/write metrics".to_string()
                })?,
        })
    } else {
        None
    };

    let python_build_ms = if all_have_python {
        Some(
            SsgRunStatistics::from_samples(&python_build_samples)
                .ok_or_else(|| "Cannot aggregate Python build metrics".to_string())?,
        )
    } else {
        None
    };

    let python_files_per_sec = if all_have_python {
        Some(
            SsgRunStatistics::from_samples(&python_throughput_samples)
                .ok_or_else(|| "Cannot aggregate Python throughput metrics".to_string())?,
        )
    } else {
        None
    };

    let ruff_vs_python_speedup = if all_have_python {
        Some(
            SsgRunStatistics::from_samples(&speedup_samples)
                .ok_or_else(|| "Cannot aggregate Ruff vs Python speedup metrics".to_string())?,
        )
    } else {
        None
    };

    Ok(SsgBenchmarkAggregateResult {
        files,
        ruff_checksum: checksum,
        ruff_build_ms: SsgRunStatistics::from_samples(&ruff_build_samples)
            .ok_or_else(|| "Cannot aggregate Ruff build metrics".to_string())?,
        ruff_files_per_sec: SsgRunStatistics::from_samples(&ruff_throughput_samples)
            .ok_or_else(|| "Cannot aggregate Ruff throughput metrics".to_string())?,
        ruff_stage_profile,
        python_build_ms,
        python_files_per_sec,
        python_stage_profile,
        ruff_vs_python_speedup,
    })
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

fn run_and_capture_with_optional_tmp_dir(
    program: &str,
    args: &[&str],
    working_dir: &Path,
    tmp_dir_override: Option<&str>,
) -> Result<String, String> {
    let mut command = Command::new(program);
    command.args(args).current_dir(working_dir);
    if let Some(tmp_dir) = tmp_dir_override {
        command.env("RUFF_BENCH_SSG_TMP_DIR", tmp_dir);
    }

    let output = command.output().map_err(|e| format!("Failed to run '{}': {}", program, e))?;

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

fn resolve_tmp_dir_override(tmp_dir: Option<&Path>) -> Result<Option<String>, String> {
    match tmp_dir {
        Some(path) => {
            let path_str = path.to_str().ok_or_else(|| {
                format!("Invalid tmp dir path (must be valid UTF-8): {}", path.display())
            })?;
            Ok(Some(path_str.to_string()))
        }
        None => Ok(None),
    }
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
    tmp_dir: Option<&Path>,
) -> Result<SsgBenchmarkResult, String> {
    if !ruff_script.exists() {
        return Err(format!("Ruff SSG benchmark script not found: {}", ruff_script.display()));
    }

    if let Some(script) = python_script {
        if !script.exists() {
            return Err(format!("Python SSG benchmark script not found: {}", script.display()));
        }
    }

    let ruff_binary_str = ruff_binary
        .to_str()
        .ok_or_else(|| format!("Invalid Ruff binary path: {}", ruff_binary.display()))?;
    let ruff_script_str = ruff_script
        .to_str()
        .ok_or_else(|| format!("Invalid Ruff script path: {}", ruff_script.display()))?;
    let working_dir = determine_workspace_root(ruff_script);
    let tmp_dir_override = resolve_tmp_dir_override(tmp_dir)?;
    let tmp_dir_override_str = tmp_dir_override.as_deref();

    let ruff_output = run_and_capture_with_optional_tmp_dir(
        ruff_binary_str,
        &["run", ruff_script_str],
        &working_dir,
        tmp_dir_override_str,
    )?;

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

        let python_output = run_and_capture_with_optional_tmp_dir(
            python_binary,
            &[python_script_str],
            &working_dir,
            tmp_dir_override_str,
        )?;
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

    #[cfg(unix)]
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    #[cfg(unix)]
    use std::sync::atomic::{AtomicU64, Ordering};
    #[cfg(unix)]
    use std::time::{SystemTime, UNIX_EPOCH};

    #[cfg(unix)]
    static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[cfg(unix)]
    fn unique_test_dir(prefix: &str) -> PathBuf {
        let mut base = std::env::current_dir().expect("current_dir should resolve");
        base.push("tmp");
        base.push("bench_ssg_harness_tests");
        fs::create_dir_all(&base).expect("bench-ssg test root should be created");

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let counter = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        base.push(format!("{}_{}_{}", prefix, timestamp, counter));
        fs::create_dir_all(&base).expect("bench-ssg test dir should be created");
        base
    }

    #[cfg(unix)]
    fn write_stub_executable(path: &Path, script_body: &str) {
        fs::write(path, script_body).expect("stub executable should be written");
        let mut perms = fs::metadata(path).expect("stub metadata should resolve").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("stub executable should be chmod +x");
    }

    #[cfg(unix)]
    fn create_basic_harness_fixture(
        prefix: &str,
        ruff_output_lines: &[&str],
        python_output_lines: Option<&[&str]>,
    ) -> (PathBuf, PathBuf, PathBuf, Option<PathBuf>) {
        let test_dir = unique_test_dir(prefix);

        let ruff_script_path = test_dir.join("stub_bench_ssg.ruff");
        fs::write(&ruff_script_path, "# stub ruff benchmark script\n")
            .expect("ruff script fixture should be written");

        let ruff_binary_path = test_dir.join("ruff_stub.sh");
        let ruff_stdout = ruff_output_lines.join("\n");
        let ruff_stub_body = format!(
            "#!/bin/sh\nif [ \"$1\" != \"run\" ]; then\n  echo \"unexpected args: $@\" >&2\n  exit 9\nfi\ncat <<'EOF'\n{}\nEOF\n",
            ruff_stdout
        );
        write_stub_executable(&ruff_binary_path, &ruff_stub_body);

        if let Some(python_lines) = python_output_lines {
            let python_script_path = test_dir.join("stub_bench_ssg.py");
            fs::write(&python_script_path, "# stub python benchmark script\n")
                .expect("python script fixture should be written");

            let python_binary_path = test_dir.join("python_stub.sh");
            let python_stdout = python_lines.join("\n");
            let python_stub_body = format!("#!/bin/sh\ncat <<'EOF'\n{}\nEOF\n", python_stdout);
            write_stub_executable(&python_binary_path, &python_stub_body);

            (ruff_binary_path, ruff_script_path, python_binary_path, Some(python_script_path))
        } else {
            (ruff_binary_path, ruff_script_path, PathBuf::new(), None)
        }
    }

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
    fn test_resolve_tmp_dir_override_none_returns_none() {
        let resolved = resolve_tmp_dir_override(None).unwrap();
        assert!(resolved.is_none());
    }

    #[test]
    fn test_resolve_tmp_dir_override_some_returns_string_path() {
        let resolved = resolve_tmp_dir_override(Some(Path::new("tmp/custom_root"))).unwrap();
        assert_eq!(resolved, Some("tmp/custom_root".to_string()));
    }

    #[cfg(unix)]
    #[test]
    fn test_resolve_tmp_dir_override_rejects_non_utf8_path() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let invalid_path = PathBuf::from(OsString::from_vec(vec![0x66, 0x6f, 0x80, 0x6f]));
        let err = resolve_tmp_dir_override(Some(invalid_path.as_path())).unwrap_err();
        assert!(err.contains("must be valid UTF-8"));
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

    #[test]
    fn test_run_statistics_from_samples_even_count() {
        let stats = SsgRunStatistics::from_samples(&[10.0, 20.0, 30.0, 40.0]).unwrap();
        assert_eq!(stats.runs, 4);
        assert!((stats.mean - 25.0).abs() < 0.0001);
        assert!((stats.median - 25.0).abs() < 0.0001);
        assert!((stats.min - 10.0).abs() < 0.0001);
        assert!((stats.max - 40.0).abs() < 0.0001);
    }

    #[test]
    fn test_run_statistics_from_samples_odd_count() {
        let stats = SsgRunStatistics::from_samples(&[8.0, 2.0, 5.0]).unwrap();
        assert_eq!(stats.runs, 3);
        assert!((stats.median - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_run_statistics_empty_samples_returns_none() {
        assert!(SsgRunStatistics::from_samples(&[]).is_none());
    }

    #[test]
    fn test_aggregate_ssg_results_without_python() {
        let runs = vec![
            SsgBenchmarkResult {
                files: 100,
                ruff_build_ms: 10.0,
                ruff_files_per_sec: 10.0,
                ruff_checksum: 42,
                ruff_stage_profile: Some(SsgStageProfile { read_ms: 4.0, render_write_ms: 6.0 }),
                python_build_ms: None,
                python_files_per_sec: None,
                python_stage_profile: None,
            },
            SsgBenchmarkResult {
                files: 100,
                ruff_build_ms: 20.0,
                ruff_files_per_sec: 5.0,
                ruff_checksum: 42,
                ruff_stage_profile: Some(SsgStageProfile { read_ms: 7.0, render_write_ms: 13.0 }),
                python_build_ms: None,
                python_files_per_sec: None,
                python_stage_profile: None,
            },
            SsgBenchmarkResult {
                files: 100,
                ruff_build_ms: 30.0,
                ruff_files_per_sec: 3.333333333,
                ruff_checksum: 42,
                ruff_stage_profile: Some(SsgStageProfile { read_ms: 10.0, render_write_ms: 20.0 }),
                python_build_ms: None,
                python_files_per_sec: None,
                python_stage_profile: None,
            },
        ];

        let summary = aggregate_ssg_results(&runs).unwrap();
        assert_eq!(summary.files, 100);
        assert_eq!(summary.ruff_checksum, 42);
        assert!((summary.ruff_build_ms.median - 20.0).abs() < 0.0001);
        assert!((summary.ruff_files_per_sec.median - 5.0).abs() < 0.0001);
        assert!(summary.python_build_ms.is_none());
        assert!(summary.python_files_per_sec.is_none());
        assert!(summary.ruff_vs_python_speedup.is_none());
        assert!(summary.ruff_stage_profile.is_some());
    }

    #[test]
    fn test_aggregate_ssg_results_with_python() {
        let runs = vec![
            SsgBenchmarkResult {
                files: 2,
                ruff_build_ms: 10.0,
                ruff_files_per_sec: 200.0,
                ruff_checksum: 7,
                ruff_stage_profile: None,
                python_build_ms: Some(20.0),
                python_files_per_sec: Some(100.0),
                python_stage_profile: None,
            },
            SsgBenchmarkResult {
                files: 2,
                ruff_build_ms: 20.0,
                ruff_files_per_sec: 100.0,
                ruff_checksum: 7,
                ruff_stage_profile: None,
                python_build_ms: Some(40.0),
                python_files_per_sec: Some(50.0),
                python_stage_profile: None,
            },
        ];

        let summary = aggregate_ssg_results(&runs).unwrap();
        assert!(summary.python_build_ms.is_some());
        assert!(summary.python_files_per_sec.is_some());

        let speedup = summary.ruff_vs_python_speedup.unwrap();
        assert!((speedup.median - 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_aggregate_ssg_results_empty_fails() {
        let err = aggregate_ssg_results(&[]).unwrap_err();
        assert!(err.contains("no runs provided"));
    }

    #[test]
    fn test_aggregate_ssg_results_inconsistent_files_fails() {
        let runs = vec![
            SsgBenchmarkResult {
                files: 10,
                ruff_build_ms: 10.0,
                ruff_files_per_sec: 10.0,
                ruff_checksum: 1,
                ruff_stage_profile: None,
                python_build_ms: None,
                python_files_per_sec: None,
                python_stage_profile: None,
            },
            SsgBenchmarkResult {
                files: 11,
                ruff_build_ms: 12.0,
                ruff_files_per_sec: 9.0,
                ruff_checksum: 1,
                ruff_stage_profile: None,
                python_build_ms: None,
                python_files_per_sec: None,
                python_stage_profile: None,
            },
        ];

        let err = aggregate_ssg_results(&runs).unwrap_err();
        assert!(err.contains("file count mismatch"));
    }

    #[test]
    fn test_aggregate_ssg_results_inconsistent_python_presence_fails() {
        let runs = vec![
            SsgBenchmarkResult {
                files: 10,
                ruff_build_ms: 10.0,
                ruff_files_per_sec: 10.0,
                ruff_checksum: 1,
                ruff_stage_profile: None,
                python_build_ms: Some(15.0),
                python_files_per_sec: Some(8.0),
                python_stage_profile: None,
            },
            SsgBenchmarkResult {
                files: 10,
                ruff_build_ms: 12.0,
                ruff_files_per_sec: 9.0,
                ruff_checksum: 1,
                ruff_stage_profile: None,
                python_build_ms: None,
                python_files_per_sec: None,
                python_stage_profile: None,
            },
        ];

        let err = aggregate_ssg_results(&runs).unwrap_err();
        assert!(err.contains("inconsistent Python comparison presence"));
    }

    #[cfg(unix)]
    #[test]
    fn test_run_ssg_benchmark_fails_when_required_ruff_metric_missing() {
        let ruff_lines =
            ["RUFF_SSG_FILES=10000", "RUFF_SSG_BUILD_MS=100.0", "RUFF_SSG_CHECKSUM=777"];
        let (ruff_binary, ruff_script, _, _) =
            create_basic_harness_fixture("missing_ruff_metric", &ruff_lines, None);

        let err = run_ssg_benchmark(ruff_binary.as_path(), ruff_script.as_path(), None, None, None)
            .unwrap_err();

        assert!(err.contains("Metric 'RUFF_SSG_FILES_PER_SEC' not found in output"));
    }

    #[cfg(unix)]
    #[test]
    fn test_run_ssg_benchmark_fails_when_required_python_metric_missing() {
        let ruff_lines = [
            "RUFF_SSG_FILES=10000",
            "RUFF_SSG_BUILD_MS=100.0",
            "RUFF_SSG_FILES_PER_SEC=1000.0",
            "RUFF_SSG_CHECKSUM=777",
        ];
        let python_lines =
            ["PYTHON_SSG_FILES=10000", "PYTHON_SSG_BUILD_MS=120.0", "PYTHON_SSG_CHECKSUM=777"];
        let (ruff_binary, ruff_script, python_binary, python_script_opt) =
            create_basic_harness_fixture("missing_python_metric", &ruff_lines, Some(&python_lines));
        let python_script = python_script_opt.expect("python script fixture should exist");

        let err = run_ssg_benchmark(
            ruff_binary.as_path(),
            ruff_script.as_path(),
            Some(python_binary.to_str().expect("python stub path should be utf8")),
            Some(python_script.as_path()),
            None,
        )
        .unwrap_err();

        assert!(err.contains("Metric 'PYTHON_SSG_FILES_PER_SEC' not found in output"));
    }

    #[cfg(unix)]
    #[test]
    fn test_run_ssg_benchmark_fails_on_python_checksum_mismatch() {
        let ruff_lines = [
            "RUFF_SSG_FILES=10000",
            "RUFF_SSG_BUILD_MS=100.0",
            "RUFF_SSG_FILES_PER_SEC=1000.0",
            "RUFF_SSG_CHECKSUM=777",
        ];
        let python_lines = [
            "PYTHON_SSG_FILES=10000",
            "PYTHON_SSG_BUILD_MS=120.0",
            "PYTHON_SSG_FILES_PER_SEC=900.0",
            "PYTHON_SSG_CHECKSUM=778",
        ];
        let (ruff_binary, ruff_script, python_binary, python_script_opt) =
            create_basic_harness_fixture("checksum_mismatch", &ruff_lines, Some(&python_lines));
        let python_script = python_script_opt.expect("python script fixture should exist");

        let err = run_ssg_benchmark(
            ruff_binary.as_path(),
            ruff_script.as_path(),
            Some(python_binary.to_str().expect("python stub path should be utf8")),
            Some(python_script.as_path()),
            None,
        )
        .unwrap_err();

        assert!(err.contains("Checksum mismatch"));
    }

    #[cfg(unix)]
    #[test]
    fn test_run_ssg_benchmark_rejects_missing_ruff_script_preflight() {
        let missing_ruff_script =
            PathBuf::from("tmp/bench_ssg_missing_ruff_script_does_not_exist.ruff");
        let err = run_ssg_benchmark(
            Path::new("/bin/echo"),
            missing_ruff_script.as_path(),
            None,
            None,
            None,
        )
        .unwrap_err();

        assert!(err.contains("Ruff SSG benchmark script not found"));
    }

    #[cfg(unix)]
    #[test]
    fn test_run_ssg_benchmark_rejects_missing_python_script_preflight() {
        let mut fixture_root = std::env::current_dir().expect("current_dir should resolve");
        fixture_root.push("tmp");
        fixture_root.push("bench_ssg_missing_python_script_preflight");
        fs::create_dir_all(&fixture_root).expect("fixture root should be created");

        let ruff_script = fixture_root.join("existing_ruff_bench.ruff");
        fs::write(&ruff_script, "# stub\n").expect("ruff script fixture should be written");

        let missing_python_script = fixture_root.join("missing_python_bench.py");

        let err = run_ssg_benchmark(
            Path::new("/bin/echo"),
            ruff_script.as_path(),
            Some("python3"),
            Some(missing_python_script.as_path()),
            None,
        )
        .unwrap_err();

        assert!(err.contains("Python SSG benchmark script not found"));
    }
}
