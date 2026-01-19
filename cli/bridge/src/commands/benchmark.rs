use anyhow::{Context, Result};
use console::style;
use std::process::Command;

pub async fn handle(params: Option<String>) -> Result<()> {
    println!("{}", style("🚀 Running benchmarks...").bold().cyan());

    // 1. Run Rust benchmarks
    println!("\n{}", style("🦀 Rust Benchmarks:").cyan());
    let mut cargo_bench = Command::new("cargo");
    cargo_bench.arg("bench");

    // Pass through extra params if any
    if let Some(p) = &params {
        cargo_bench.arg(p);
    }

    let status = cargo_bench.status().context("Failed to run cargo bench")?;
    if !status.success() {
        println!(
            "{}",
            style("⚠ Rust benchmarks failed or returned non-zero exit code.").yellow()
        );
    }

    // 2. Run Python benchmarks (if pytest-benchmark is present)
    // We check for `python` directory or `pyproject.toml`
    let has_python =
        std::path::Path::new("pyproject.toml").exists() || std::path::Path::new("python").exists();

    if has_python {
        println!("\n{}", style("🐍 Python Benchmarks:").cyan());
        // Try running pytest
        // We assume the user has set up their venv.
        // We look for 'pytest' in path.
        if Command::new("pytest").arg("--version").output().is_ok() {
            let mut pytest = Command::new("pytest");
            // Add some benchmark flags if using pytest-benchmark plugin,
            // but user might not have it. Safest is just running pytest.
            // If we want to be fancy, we can check for the plugin.
            // For now, let's just interpret 'params' as args for pytest too if easy,
            // but usually cargo bench args differ from pytest args.
            // So we just run default pytest for now, maybe in 'benchmarks' folder?

            // Common convention: tests/benchmarks or similar.
            // BridgeRust structure: check if `benchmarks` dir exists?
            if std::path::Path::new("benchmarks").exists() {
                pytest.arg("benchmarks");
            }

            let py_status = pytest.status().context("Failed to run pytest")?;
            if !py_status.success() {
                println!("{}", style("⚠ Python benchmarks failed.").yellow());
            }
        } else {
            println!(
                "  {}",
                style("pytest not found. Skipping Python benchmarks.").yellow()
            );
        }
    }

    Ok(())
}
