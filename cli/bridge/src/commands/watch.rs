use anyhow::{Context, Result};
use console::style;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

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

pub async fn handle(target: String, test: bool) -> Result<()> {
    println!("{}", style("👀 Watching for changes...").bold().cyan());
    println!("  Press Ctrl+C to stop\n");

    let project_root = find_project_root()?;
    println!("  Project root: {}", project_root.display());

    // Create a channel to receive file system events
    let (tx, rx) = mpsc::channel();

    // Create a watcher
    let mut watcher = notify::recommended_watcher(tx).context("Failed to create file watcher")?;

    // Watch the src directory
    let src_dir = project_root.join("src");
    if src_dir.exists() {
        watcher
            .watch(&src_dir, RecursiveMode::Recursive)
            .context("Failed to watch src directory")?;
        println!("  {} Watching: {}", style("✓").green(), src_dir.display());
    }

    // Watch Cargo.toml
    let cargo_toml = project_root.join("Cargo.toml");
    if cargo_toml.exists() {
        watcher
            .watch(&cargo_toml, RecursiveMode::NonRecursive)
            .context("Failed to watch Cargo.toml")?;
        println!(
            "  {} Watching: {}",
            style("✓").green(),
            cargo_toml.display()
        );
    }

    // Watch target-specific config files
    let python_dir = project_root.join("python");
    let nodejs_dir = project_root.join("nodejs");

    let targets: Vec<&str> = match target.as_str() {
        "all" => vec!["python", "nodejs"],
        "python" => vec!["python"],
        "nodejs" => vec!["nodejs"],
        _ => anyhow::bail!(
            "Invalid target: {}. Use 'python', 'nodejs', or 'all'",
            target
        ),
    };

    for target_name in &targets {
        match *target_name {
            "python" => {
                let pyproject = python_dir.join("pyproject.toml");
                if pyproject.exists() {
                    watcher
                        .watch(&pyproject, RecursiveMode::NonRecursive)
                        .context("Failed to watch pyproject.toml")?;
                    println!("  {} Watching: {}", style("✓").green(), pyproject.display());
                }
            }
            "nodejs" => {
                let package_json = nodejs_dir.join("package.json");
                if package_json.exists() {
                    watcher
                        .watch(&package_json, RecursiveMode::NonRecursive)
                        .context("Failed to watch package.json")?;
                    println!(
                        "  {} Watching: {}",
                        style("✓").green(),
                        package_json.display()
                    );
                }
            }
            _ => {}
        }
    }

    println!("\n{} Waiting for changes...\n", style("→").cyan());

    let mut last_build_time = Instant::now();
    let debounce_duration = Duration::from_millis(500);

    loop {
        // Check for file system events
        match rx.try_recv() {
            Ok(Ok(Event {
                kind: EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_),
                paths,
                ..
            })) => {
                // Filter out non-Rust files and build artifacts
                let relevant_change = paths.iter().any(|path| {
                    path.extension()
                        .map(|ext| ext == "rs" || ext == "toml" || ext == "json")
                        .unwrap_or(false)
                });

                if relevant_change && last_build_time.elapsed() > debounce_duration {
                    last_build_time = Instant::now();

                    println!(
                        "{}",
                        style("🔄 Change detected, rebuilding...").bold().yellow()
                    );

                    // Build the target(s)
                    for target_name in &targets {
                        if let Err(e) = build_target(target_name, &project_root).await {
                            eprintln!("  {} Build failed: {}", style("✗").red(), e);
                        } else {
                            println!("  {} {} build completed", style("✓").green(), target_name);
                        }
                    }

                    // Run tests if requested
                    if test {
                        println!("{}", style("🧪 Running tests...").bold().cyan());
                        for target_name in &targets {
                            if let Err(e) = test_target(target_name, &project_root).await {
                                eprintln!("  {} Test failed: {}", style("✗").red(), e);
                            } else {
                                println!("  {} {} tests passed", style("✓").green(), target_name);
                            }
                        }
                    }

                    println!("\n{} Waiting for changes...\n", style("→").cyan());
                }
            }
            Ok(Ok(_)) => {} // Ignore other event types
            Ok(Err(e)) => {
                eprintln!("  {} File watcher error: {}", style("⚠").yellow(), e);
            }
            Err(mpsc::TryRecvError::Empty) => {} // No events yet
            Err(mpsc::TryRecvError::Disconnected) => {
                eprintln!("{}", style("File watcher disconnected").red());
                break;
            }
        }

        // Small sleep to avoid busy waiting
        sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}

async fn build_target(target: &str, project_root: &Path) -> Result<()> {
    match target {
        "python" => {
            let python_dir = project_root.join("python");
            let mut cmd = Command::new("maturin");
            cmd.current_dir(&python_dir);
            cmd.arg("develop");
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());

            let status = cmd.status().context("Failed to run maturin develop")?;
            if !status.success() {
                anyhow::bail!("Python build failed");
            }
        }
        "nodejs" => {
            let nodejs_dir = project_root.join("nodejs");
            let mut cmd = Command::new("npm");
            cmd.current_dir(&nodejs_dir);
            cmd.arg("run");
            cmd.arg("build");
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());

            let status = cmd.status().context("Failed to run npm build")?;
            if !status.success() {
                anyhow::bail!("Node.js build failed");
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

async fn test_target(target: &str, project_root: &PathBuf) -> Result<()> {
    match target {
        "python" => {
            let mut cmd = Command::new("python");
            cmd.arg("-m");
            cmd.arg("pytest");
            cmd.current_dir(project_root);
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());

            let status = cmd.status().context("Failed to run pytest")?;
            if !status.success() {
                anyhow::bail!("Python tests failed");
            }
        }
        "nodejs" => {
            let nodejs_dir = project_root.join("nodejs");
            let mut cmd = Command::new("npm");
            cmd.current_dir(&nodejs_dir);
            cmd.arg("test");
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());

            let status = cmd.status().context("Failed to run npm test")?;
            if !status.success() {
                anyhow::bail!("Node.js tests failed");
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
