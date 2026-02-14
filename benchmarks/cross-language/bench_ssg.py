#!/usr/bin/env python3

import shutil
import time
from pathlib import Path


def run_ssg_benchmark() -> tuple[int, float, float, int, float, float]:
    file_count = 10_000
    workspace_root = Path(__file__).resolve().parents[2]
    tmp_root = workspace_root / "tmp"
    tmp_root.mkdir(parents=True, exist_ok=True)
    base_dir = tmp_root / f"ruff_ssg_bench_py_{time.time_ns()}"
    input_dir = base_dir / "input"
    output_dir = base_dir / "output"
    input_dir.mkdir(parents=True, exist_ok=True)
    output_dir.mkdir(parents=True, exist_ok=True)

    try:
        for index in range(file_count):
            source_path = input_dir / f"post_{index}.md"
            source_path.write_text(
                f"# Post {index}\n\nGenerated page {index}",
                encoding="utf-8",
            )

        start = time.perf_counter()

        pages = []
        for index in range(file_count):
            source_path = input_dir / f"post_{index}.md"
            pages.append(source_path.read_text(encoding="utf-8"))
        read_stage_ms = (time.perf_counter() - start) * 1000.0

        checksum = 0
        for index, page in enumerate(pages):
            html = (
                f"<html><body><h1>Post {index}</h1>"
                f"<article>{page}</article></body></html>"
            )
            checksum += len(html)
            output_path = output_dir / f"post_{index}.html"
            output_path.write_text(html, encoding="utf-8")

        render_write_stage_ms = ((time.perf_counter() - start) * 1000.0) - read_stage_ms

        elapsed_ms = (time.perf_counter() - start) * 1000.0
        files_per_sec = (file_count * 1000.0) / elapsed_ms if elapsed_ms > 0 else 0.0

        return (
            file_count,
            elapsed_ms,
            files_per_sec,
            checksum,
            read_stage_ms,
            render_write_stage_ms,
        )
    finally:
        shutil.rmtree(base_dir, ignore_errors=True)


def main() -> None:
    (
        files,
        elapsed_ms,
        files_per_sec,
        checksum,
        read_stage_ms,
        render_write_stage_ms,
    ) = run_ssg_benchmark()

    print(f"PYTHON_SSG_FILES={files}")
    print(f"PYTHON_SSG_BUILD_MS={elapsed_ms:.6f}")
    print(f"PYTHON_SSG_FILES_PER_SEC={files_per_sec:.6f}")
    print(f"PYTHON_SSG_CHECKSUM={checksum}")
    print(f"PYTHON_SSG_READ_MS={read_stage_ms:.6f}")
    print(f"PYTHON_SSG_RENDER_WRITE_MS={render_write_stage_ms:.6f}")


if __name__ == "__main__":
    main()
