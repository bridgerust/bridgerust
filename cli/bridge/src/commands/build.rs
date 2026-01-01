use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;

/// Get the cache directory for BridgeRust builds
fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?
        .join("bridgerust");

    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

/// Get the last modified time of a file or directory
fn get_last_modified(path: &PathBuf) -> Result<SystemTime> {
    let metadata = fs::metadata(path)?;
    metadata
        .modified()
        .or_else(|_| metadata.created())
        .context("Could not get file modification time")
}

/// Check if a build is up to date based on source file timestamps
fn is_build_up_to_date(project_root: &Path, target: &str, cache_file: &PathBuf) -> bool {
    // Check if cache file exists
    if !cache_file.exists() {
        return false;
    }

    // Get cache timestamp
    let cache_time = match get_last_modified(cache_file) {
        Ok(time) => time,
        Err(_) => return false,
    };

    // Check source files
    let src_dir = project_root.join("src");
    if src_dir.exists() {
        if let Ok(entries) = fs::read_dir(&src_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map(|e| e == "rs").unwrap_or(false) {
                    if let Ok(src_time) = get_last_modified(&entry.path()) {
                        if src_time > cache_time {
                            return false;
                        }
                    }
                }
            }
        }
    }

    // Check Cargo.toml
    let cargo_toml = project_root.join("Cargo.toml");
    if cargo_toml.exists() {
        if let Ok(cargo_time) = get_last_modified(&cargo_toml) {
            if cargo_time > cache_time {
                return false;
            }
        }
    }

    // Check target-specific config files
    match target {
        "python" => {
            let pyproject = project_root.join("python").join("pyproject.toml");
            if pyproject.exists() {
                if let Ok(py_time) = get_last_modified(&pyproject) {
                    if py_time > cache_time {
                        return false;
                    }
                }
            }
        }
        "nodejs" => {
            let package_json = project_root.join("nodejs").join("package.json");
            if package_json.exists() {
                if let Ok(pkg_time) = get_last_modified(&package_json) {
                    if pkg_time > cache_time {
                        return false;
                    }
                }
            }
        }
        _ => {}
    }

    true
}

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

/// Find Python build directory (contains pyproject.toml)
fn find_python_dir(project_root: &Path) -> Option<PathBuf> {
    // Check common locations
    let candidates = vec![
        project_root.join("python"),
        project_root.join("bindings").join("python"),
        project_root.to_path_buf(),
    ];

    candidates
        .into_iter()
        .find(|candidate| candidate.join("pyproject.toml").exists())
}

/// Find Node.js build directory (contains package.json with napi config)
fn find_nodejs_dir(project_root: &Path) -> Option<PathBuf> {
    // Check common locations
    let candidates = vec![
        project_root.join("nodejs"),
        project_root.join("bindings").join("node"),
        project_root.to_path_buf(),
    ];

    for candidate in candidates {
        let package_json = candidate.join("package.json");
        if package_json.exists() {
            // Check if it has napi config (basic check)
            if let Ok(content) = std::fs::read_to_string(&package_json) {
                if content.contains("\"napi\"") || content.contains("napi-rs") {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

pub async fn handle(target: String, release: bool) -> Result<()> {
    println!(
        "{}",
        style("🔨 Building BridgeRust project...").bold().cyan()
    );

    let project_root = find_project_root()?;
    println!("  Project root: {}", project_root.display());

    let targets: Vec<&str> = match target.as_str() {
        "all" => vec!["python", "nodejs"],
        "python" => vec!["python"],
        "nodejs" => vec!["nodejs"],
        _ => anyhow::bail!(
            "Invalid target: {}. Use 'python', 'nodejs', or 'all'",
            target
        ),
    };

    // Check build cache (only for debug builds)
    let use_cache = !release;
    let cache_dir = if use_cache {
        Some(get_cache_dir()?)
    } else {
        None
    };

    // Build targets in parallel when building "all"
    if targets.len() > 1 {
        println!("  Building {} targets in parallel...", targets.len());
        println!("  {} Python build starting...", style("→").cyan());
        println!("  {} Node.js build starting...", style("→").cyan());

        let start_time = std::time::Instant::now();
        let cache_ref = cache_dir.as_ref();
        let (python_result, nodejs_result) = tokio::join!(
            build_target("python", release, &project_root, true, cache_ref),
            build_target("nodejs", release, &project_root, true, cache_ref)
        );
        let elapsed = start_time.elapsed();

        // Check both results and report
        match python_result {
            Ok(_) => println!("  {} Python build completed", style("✓").green()),
            Err(e) => {
                eprintln!("  {} Python build failed: {}", style("✗").red(), e);
                return Err(e);
            }
        }

        match nodejs_result {
            Ok(_) => println!("  {} Node.js build completed", style("✓").green()),
            Err(e) => {
                eprintln!("  {} Node.js build failed: {}", style("✗").red(), e);
                return Err(e);
            }
        }

        println!(
            "  {} Total build time: {:.2}s",
            style("⏱").cyan(),
            elapsed.as_secs_f64()
        );
    } else {
        // Single target - build sequentially
        let cache_ref = cache_dir.as_ref();
        for target in targets {
            build_target(target, release, &project_root, false, cache_ref).await?;
        }
    }

    println!("\n{}", style("✅ Build completed!").bold().green());
    Ok(())
}

async fn build_target(
    target: &str,
    release: bool,
    project_root: &PathBuf,
    parallel: bool,
    cache_dir: Option<&PathBuf>,
) -> Result<()> {
    // Check cache first (only for debug builds)
    if let Some(cache) = cache_dir {
        let cache_file = cache.join(format!("{}.cache", target));
        if is_build_up_to_date(project_root, target, &cache_file) {
            if !parallel {
                println!(
                    "  {} {} bindings (cached, up to date)",
                    style("✓").green(),
                    target
                );
            }
            return Ok(());
        }
    }

    let start_time = std::time::Instant::now();
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Building {} bindings...", target));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    match target {
        "python" => {
            let python_dir = find_python_dir(project_root)
                .context("Could not find Python build directory (looking for pyproject.toml)")?;

            println!("  Python directory: {}", python_dir.display());

            // Check if maturin is installed
            let maturin_check = Command::new("maturin").arg("--version").output();

            if maturin_check.is_err() {
                anyhow::bail!("maturin is not installed. Install it with: pip install maturin");
            }

            let mut cmd = Command::new("maturin");
            cmd.current_dir(&python_dir);
            cmd.arg("build");

            if release {
                cmd.arg("--release");
            }

            // Maturin reads features from pyproject.toml, but we can override
            cmd.arg("--features").arg("python");

            // Show output in real-time
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());

            pb.finish_and_clear();

            let status = cmd.status().context("Failed to run maturin")?;

            if !status.success() {
                anyhow::bail!("Python build failed");
            }

            // Update cache
            if let Some(cache) = cache_dir {
                let cache_file = cache.join("python.cache");
                let _ = fs::File::create(&cache_file);
            }

            let elapsed = start_time.elapsed();
            if !parallel {
                println!(
                    "  {} Python bindings built ({:.2}s)",
                    style("✓").green(),
                    elapsed.as_secs_f64()
                );
            }
        }
        "nodejs" => {
            let nodejs_dir = find_nodejs_dir(project_root)
                .context("Could not find Node.js build directory (looking for package.json with napi config)")?;

            println!("  Node.js directory: {}", nodejs_dir.display());

            // Check if @napi-rs/cli is available via npm
            let napi_check = Command::new("npx")
                .arg("--yes")
                .arg("@napi-rs/cli")
                .arg("--version")
                .current_dir(&nodejs_dir)
                .output();

            // Try using napi-rs CLI if available
            if napi_check.is_ok() {
                let mut cmd = Command::new("npx");
                cmd.current_dir(&nodejs_dir);
                cmd.arg("--yes");
                cmd.arg("@napi-rs/cli");
                cmd.arg("build");
                cmd.arg("--platform");

                if release {
                    cmd.arg("--release");
                }

                cmd.stdout(Stdio::inherit());
                cmd.stderr(Stdio::inherit());

                pb.finish_and_clear();

                let status = cmd.status().context("Failed to run napi-rs CLI")?;

                if !status.success() {
                    anyhow::bail!("Node.js build failed");
                }

                // Update cache
                if let Some(cache) = cache_dir {
                    let cache_file = cache.join("nodejs.cache");
                    let _ = fs::File::create(&cache_file);
                }
            } else {
                // Fallback to cargo build
                println!("  Note: Using cargo build (napi-rs CLI not found)");

                let mut cmd = Command::new("cargo");
                cmd.current_dir(project_root);
                cmd.arg("build");

                if release {
                    cmd.arg("--release");
                }

                cmd.arg("--features").arg("nodejs");

                cmd.stdout(Stdio::inherit());
                cmd.stderr(Stdio::inherit());

                pb.finish_and_clear();

                let status = cmd.status().context("Failed to run cargo")?;

                if !status.success() {
                    anyhow::bail!("Node.js build failed");
                }

                // Update cache
                if let Some(cache) = cache_dir {
                    let cache_file = cache.join("nodejs.cache");
                    let _ = fs::File::create(&cache_file);
                }
            }

            let elapsed = start_time.elapsed();
            if !parallel {
                println!(
                    "  {} Node.js bindings built ({:.2}s)",
                    style("✓").green(),
                    elapsed.as_secs_f64()
                );
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
