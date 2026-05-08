use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "ruff_{}_{}_{}_{}",
        prefix,
        std::process::id(),
        nanos,
        counter
    ));
    fs::create_dir_all(&path).expect("failed to create temp directory");
    path
}

fn ruff_binary() -> String {
    env!("CARGO_BIN_EXE_ruff").to_string()
}

fn run_ruff(args: &[&str], current_dir: &Path) -> Output {
    Command::new(ruff_binary())
        .current_dir(current_dir)
        .args(args)
        .output()
        .expect("failed to execute ruff binary")
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8")
}

fn escape_ruff_string(raw: &str) -> String {
    raw.replace('\\', "\\\\").replace('"', "\\\"")
}

fn run_unzip_script_with_archive<F>(prefix: &str, archive_builder: F) -> (PathBuf, PathBuf, Output)
where
    F: FnOnce(&Path),
{
    let project_root = unique_temp_dir(prefix);
    let zip_path = project_root.join("payload.zip");
    let output_dir = project_root.join("unzipped");
    archive_builder(&zip_path);

    let script_path = project_root.join("boundary.ruff");
    let script_source = format!(
        "unzip(\"{}\", \"{}\")\n",
        escape_ruff_string(zip_path.to_str().expect("zip path should be utf-8")),
        escape_ruff_string(output_dir.to_str().expect("output path should be utf-8")),
    );
    fs::write(&script_path, script_source).expect("failed to write unzip script");

    let output = run_ruff(
        &["run", script_path.to_str().expect("script path should be utf-8"), "--interpreter"],
        &project_root,
    );

    (project_root, output_dir, output)
}

fn assert_unzip_failure(output: &Output, expected_runtime_error: &str) {
    assert_eq!(
        output.status.code(),
        Some(1),
        "expected unzip boundary failure with exit code 1, got status={:?}, stdout={}, stderr={}",
        output.status.code(),
        stdout_text(output),
        stderr_text(output)
    );

    let combined_output = format!("{}\n{}", stdout_text(output), stderr_text(output));
    assert!(
        combined_output.contains(expected_runtime_error),
        "expected runtime error text '{}' in output, got stdout={} stderr={}",
        expected_runtime_error,
        stdout_text(output),
        stderr_text(output)
    );
}

fn write_zip_file_entry(
    writer: &mut ZipWriter<fs::File>,
    entry_name: &str,
    contents: &[u8],
    unix_mode: Option<u32>,
) {
    let mut options = FileOptions::default().compression_method(CompressionMethod::Deflated);
    if let Some(mode) = unix_mode {
        options = options.unix_permissions(mode);
    }
    writer.start_file(entry_name, options).expect("failed to start zip file entry");
    writer.write_all(contents).expect("failed to write zip file entry contents");
}

fn create_zip_archive<F>(zip_path: &Path, builder: F)
where
    F: FnOnce(&mut ZipWriter<fs::File>),
{
    let file = fs::File::create(zip_path).expect("failed to create zip archive");
    let mut writer = ZipWriter::new(file);
    builder(&mut writer);
    writer.finish().expect("failed to finalize zip archive");
}

fn mark_first_zip_entry_as_symlink(zip_path: &Path) {
    let mut archive_bytes = fs::read(zip_path).expect("failed to read zip archive bytes");
    let central_directory_signature = [0x50, 0x4b, 0x01, 0x02];
    let Some(header_start) =
        archive_bytes.windows(4).position(|window| window == central_directory_signature)
    else {
        panic!("expected central directory header in zip archive");
    };

    // Mark as Unix host so unix_mode() is populated by zip::read::ZipFile.
    archive_bytes[header_start + 5] = 3;

    // Central directory external attributes field (offset 38) stores unix mode in the upper 16 bits.
    let symlink_mode_external_attrs = (0o120777_u32) << 16;
    archive_bytes[header_start + 38..header_start + 42]
        .copy_from_slice(&symlink_mode_external_attrs.to_le_bytes());

    fs::write(zip_path, archive_bytes).expect("failed to write patched zip archive bytes");
}

fn assert_runtime_boundary_failure(script_source: &str, expected_runtime_error: &str) {
    let project_root = unique_temp_dir("native_api_security_boundary");
    let script_path = project_root.join("boundary.ruff");
    fs::write(&script_path, script_source).expect("failed to write script");

    let output = run_ruff(
        &["run", script_path.to_str().expect("script path should be utf-8"), "--interpreter"],
        &project_root,
    );

    assert_eq!(
        output.status.code(),
        Some(1),
        "expected runtime misuse to exit with code 1, got status={:?}, stdout={}, stderr={}",
        output.status.code(),
        stdout_text(&output),
        stderr_text(&output)
    );

    let combined_output = format!("{}\n{}", stdout_text(&output), stderr_text(&output));
    assert!(
        combined_output.contains(expected_runtime_error),
        "expected runtime error text '{}' in output, got stdout={} stderr={}",
        expected_runtime_error,
        stdout_text(&output),
        stderr_text(&output)
    );
}

#[test]
fn process_native_api_misuse_reports_deterministic_error() {
    assert_runtime_boundary_failure("execute(123)\n", "execute() requires a string command");
}

#[test]
fn network_native_api_misuse_reports_deterministic_error() {
    assert_runtime_boundary_failure(
        "tcp_receive(1, 10)\n",
        "tcp_receive requires (TcpStream, int_size) arguments",
    );
}

#[test]
fn filesystem_native_api_misuse_reports_deterministic_error() {
    assert_runtime_boundary_failure("write_file(1, 2)\n", "write_file requires string arguments");
}

#[test]
fn crypto_native_api_misuse_reports_deterministic_error() {
    assert_runtime_boundary_failure(
        "rsa_generate_keypair(1024)\n",
        "RSA key size must be 2048 or 4096 bits",
    );
}

#[test]
fn database_native_api_misuse_reports_deterministic_error() {
    assert_runtime_boundary_failure(
        "db_connect(\"sqlite\")\n",
        "db_connect requires database type ('sqlite'|'postgres'|'mysql') and connection string",
    );
}

#[test]
fn unzip_rejects_parent_traversal_entries() {
    let (project_root, _, output) =
        run_unzip_script_with_archive("native_api_unzip_parent_traversal", |zip_path| {
            create_zip_archive(zip_path, |writer| {
                write_zip_file_entry(writer, "../escape.txt", b"escape", None);
            });
        });

    assert_unzip_failure(&output, "parent directory traversal component");
    assert!(
        !project_root.join("escape.txt").exists(),
        "zip traversal entry should not write files outside extraction root"
    );
}

#[test]
fn unzip_rejects_absolute_entries() {
    let (_, _, output) =
        run_unzip_script_with_archive("native_api_unzip_absolute_path", |zip_path| {
            create_zip_archive(zip_path, |writer| {
                write_zip_file_entry(writer, "/tmp/escape.txt", b"escape", None);
            });
        });

    assert_unzip_failure(&output, "absolute path");
}

#[test]
fn unzip_rejects_windows_drive_prefixed_entries() {
    let (_, _, output) =
        run_unzip_script_with_archive("native_api_unzip_drive_prefix", |zip_path| {
            create_zip_archive(zip_path, |writer| {
                write_zip_file_entry(writer, "C:/escape.txt", b"escape", None);
            });
        });

    assert_unzip_failure(&output, "drive-prefixed path");
}

#[test]
fn unzip_rejects_null_byte_entries() {
    let (_, _, output) = run_unzip_script_with_archive("native_api_unzip_null_byte", |zip_path| {
        create_zip_archive(zip_path, |writer| {
            write_zip_file_entry(writer, "bad\0name.txt", b"escape", None);
        });
    });

    assert_unzip_failure(&output, "null byte");
}

#[test]
fn unzip_rejects_symlink_entries() {
    let (_, _, output) = run_unzip_script_with_archive("native_api_unzip_symlink", |zip_path| {
        create_zip_archive(zip_path, |writer| {
            write_zip_file_entry(writer, "symlink-entry", b"target.txt", None);
        });
        mark_first_zip_entry_as_symlink(zip_path);
    });

    assert_unzip_failure(&output, "symbolic links are not allowed");
}

#[test]
fn unzip_rejects_archives_exceeding_single_entry_limit() {
    let (_, _, output) =
        run_unzip_script_with_archive("native_api_unzip_single_limit", |zip_path| {
            create_zip_archive(zip_path, |writer| {
                let oversized = vec![b'x'; 17 * 1024 * 1024];
                write_zip_file_entry(writer, "oversized.bin", &oversized, None);
            });
        });

    assert_unzip_failure(&output, "exceeds maximum per-entry size");
}

#[test]
fn unzip_rejects_archives_exceeding_total_size_limit() {
    let (_, _, output) =
        run_unzip_script_with_archive("native_api_unzip_total_limit", |zip_path| {
            create_zip_archive(zip_path, |writer| {
                let payload = vec![b'y'; 14 * 1024 * 1024];
                for index in 0..5 {
                    write_zip_file_entry(writer, &format!("bulk-{}.bin", index), &payload, None);
                }
            });
        });

    assert_unzip_failure(&output, "exceeds maximum total extraction size");
}

#[test]
fn unzip_rejects_archives_exceeding_entry_count_limit() {
    let (_, _, output) =
        run_unzip_script_with_archive("native_api_unzip_entry_count_limit", |zip_path| {
            create_zip_archive(zip_path, |writer| {
                for index in 0..1025 {
                    write_zip_file_entry(writer, &format!("entry-{}.txt", index), b"ok", None);
                }
            });
        });

    assert_unzip_failure(&output, "exceeds maximum entry count");
}

#[test]
fn unzip_extracts_safe_nested_entries() {
    let (project_root, output_dir, output) =
        run_unzip_script_with_archive("native_api_unzip_safe_nested", |zip_path| {
            create_zip_archive(zip_path, |writer| {
                write_zip_file_entry(writer, "safe/nested/file.txt", b"hello", None);
                write_zip_file_entry(writer, "safe/nested/second.txt", b"world", None);
            });
        });

    assert_eq!(
        output.status.code(),
        Some(0),
        "expected unzip success, got status={:?}, stdout={}, stderr={}",
        output.status.code(),
        stdout_text(&output),
        stderr_text(&output)
    );

    let first_file = output_dir.join("safe/nested/file.txt");
    let second_file = output_dir.join("safe/nested/second.txt");
    assert!(
        first_file.exists() && second_file.exists(),
        "expected safe nested files to be extracted under output directory; output root={} stdout={} stderr={}",
        project_root.display(),
        stdout_text(&output),
        stderr_text(&output)
    );
    assert_eq!(fs::read_to_string(first_file).expect("expected first extracted file"), "hello");
    assert_eq!(fs::read_to_string(second_file).expect("expected second extracted file"), "world");
}
