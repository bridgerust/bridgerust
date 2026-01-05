use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Find the project root by looking for Cargo.toml or bridgerust.toml
fn find_project_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        let bridgerust_toml = current.join("bridgerust.toml");

        if cargo_toml.exists() || bridgerust_toml.exists() {
            return Ok(current);
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => anyhow::bail!("Could not find project root (Cargo.toml or bridgerust.toml)"),
        }
    }
}

pub async fn handle(target: String) -> Result<()> {
    println!("{}", style("🧪 Running tests...").bold().cyan());

    let project_root = find_project_root()?;
    println!("  Project root: {}", project_root.display());

    let targets: Vec<&str> = match target.as_str() {
        "all" => vec!["rust", "python", "nodejs"],
        "python" => vec!["python"],
        "nodejs" => vec!["nodejs"],
        "rust" => vec!["rust"],
        _ => anyhow::bail!(
            "Invalid target: {}. Use 'python', 'nodejs', 'rust', or 'all'",
            target
        ),
    };

    let mut failed = Vec::new();

    for target in &targets {
        match test_target(target, &project_root).await {
            Ok(_) => {}
            Err(e) => {
                failed.push((*target, e));
            }
        }
    }

    if !failed.is_empty() {
        eprintln!("\n{}", style("❌ Some tests failed:").bold().red());
        for (target, error) in &failed {
            eprintln!("  {} {}: {}", style("✗").red(), target, error);
        }
        anyhow::bail!("Tests failed");
    }

    println!("\n{}", style("✅ All tests passed!").bold().green());
    Ok(())
}

async fn test_target(target: &str, project_root: &PathBuf) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Running {} tests...", target));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    match target {
        "rust" => {
            pb.finish_and_clear();
            let mut cmd = Command::new("cargo");
            cmd.current_dir(project_root);
            cmd.arg("test");
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());

            let status = cmd.status().context("Failed to run cargo test")?;

            if !status.success() {
                anyhow::bail!("Rust tests failed");
            }
            println!("  {} Rust tests passed", style("✓").green());
        }
        "python" => {
            // Check if pytest is available
            let pytest_check = Command::new("python")
                .arg("-m")
                .arg("pytest")
                .arg("--version")
                .output();

            if pytest_check.is_err() {
                // Try python3
                let pytest3_check = Command::new("python3")
                    .arg("-m")
                    .arg("pytest")
                    .arg("--version")
                    .output();

                if pytest3_check.is_err() {
                    anyhow::bail!("pytest is not installed. Install it with: pip install pytest");
                }
            }

            // Find test files
            let test_dirs = vec![
                project_root.join("tests"),
                project_root.join("python").join("tests"),
                project_root.join("python"),
            ];

            let mut test_files = Vec::new();
            for test_dir in &test_dirs {
                if test_dir.exists()
                    && let Ok(entries) = std::fs::read_dir(test_dir)
                {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|e| e == "py").unwrap_or(false)
                            && path
                                .file_name()
                                .map(|n| n.to_string_lossy().starts_with("test_"))
                                .unwrap_or(false)
                        {
                            test_files.push(path);
                        }
                    }
                }
            }

            pb.finish_and_clear();

            if test_files.is_empty() {
                println!("  {} No Python test files found", style("⚠").yellow());
                return Ok(());
            }

            let mut cmd = Command::new("python");
            cmd.arg("-m").arg("pytest");
            for test_file in &test_files {
                cmd.arg(test_file);
            }
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());

            let status = cmd.status().context("Failed to run pytest")?;

            if !status.success() {
                anyhow::bail!("Python tests failed");
            }
            println!("  {} Python tests passed", style("✓").green());
        }
        "nodejs" => {
            let nodejs_dir = project_root.join("nodejs");

            // Check if package.json exists and has test script
            let package_json = nodejs_dir.join("package.json");
            let has_test_script = if package_json.exists() {
                if let Ok(content) = std::fs::read_to_string(&package_json) {
                    content.contains("\"test\"") || content.contains("\"tests\"")
                } else {
                    false
                }
            } else {
                false
            };

            pb.finish_and_clear();

            if has_test_script {
                // Use npm test
                let mut cmd = Command::new("npm");
                cmd.current_dir(&nodejs_dir);
                cmd.arg("test");
                cmd.stdout(Stdio::inherit());
                cmd.stderr(Stdio::inherit());

                let status = cmd.status().context("Failed to run npm test")?;

                if !status.success() {
                    anyhow::bail!("Node.js tests failed");
                }
                println!("  {} Node.js tests passed", style("✓").green());
            } else {
                // Look for test files directly
                let test_files = vec![
                    nodejs_dir.join("test_bindings.js"),
                    nodejs_dir.join("tests").join("test_bindings.js"),
                    nodejs_dir.join("test").join("test_bindings.js"),
                ];

                let mut found_test = None;
                for test_file in &test_files {
                    if test_file.exists() {
                        found_test = Some(test_file.clone());
                        break;
                    }
                }

                if let Some(test_file) = found_test {
                    let mut cmd = Command::new("node");
                    cmd.arg(&test_file);
                    cmd.stdout(Stdio::inherit());
                    cmd.stderr(Stdio::inherit());

                    let status = cmd.status().context("Failed to run Node.js tests")?;

                    if !status.success() {
                        anyhow::bail!("Node.js tests failed");
                    }
                    println!("  {} Node.js tests passed", style("✓").green());
                } else {
                    println!("  {} No Node.js test files found", style("⚠").yellow());
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
