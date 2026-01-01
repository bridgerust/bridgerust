use anyhow::Result;
use console::style;
use std::fs;
use std::path::PathBuf;

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

/// Get the cache directory for BridgeRust builds
fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?
        .join("bridgerust");
    Ok(cache_dir)
}

pub async fn handle(target: Option<String>, cache: bool) -> Result<()> {
    println!(
        "{}",
        style("🧹 Cleaning BridgeRust project...").bold().cyan()
    );

    let project_root = find_project_root()?;
    println!("  Project root: {}", project_root.display());

    let targets: Vec<&str> = match target.as_deref() {
        Some("all") | None => vec!["python", "nodejs", "rust"],
        Some("python") => vec!["python"],
        Some("nodejs") => vec!["nodejs"],
        Some("rust") => vec!["rust"],
        Some(other) => anyhow::bail!(
            "Invalid target: {}. Use 'python', 'nodejs', 'rust', or 'all'",
            other
        ),
    };

    let mut cleaned = Vec::new();

    for target in &targets {
        match *target {
            "python" => {
                let python_dir = project_root.join("python");
                let wheels_dir = python_dir.join("target").join("wheels");
                let dist_dir = python_dir.join("dist");

                if wheels_dir.exists() {
                    fs::remove_dir_all(&wheels_dir)?;
                    cleaned.push(format!("Python wheels ({})", wheels_dir.display()));
                }

                if dist_dir.exists() {
                    fs::remove_dir_all(&dist_dir)?;
                    cleaned.push(format!("Python dist ({})", dist_dir.display()));
                }

                // Clean maturin build artifacts
                let maturin_target = python_dir.join("target");
                if maturin_target.exists() && maturin_target.join("wheels").exists() {
                    let wheels = maturin_target.join("wheels");
                    if wheels.exists() {
                        fs::remove_dir_all(&wheels)?;
                        cleaned.push(format!("Maturin wheels ({})", wheels.display()));
                    }
                }
            }
            "nodejs" => {
                let nodejs_dir = project_root.join("nodejs");
                let index_node = nodejs_dir.join("index.node");
                let index_js = nodejs_dir.join("index.js");
                let index_d_ts = nodejs_dir.join("index.d.ts");

                if index_node.exists() {
                    fs::remove_file(&index_node)?;
                    cleaned.push("Node.js index.node".to_string());
                }

                if index_js.exists() {
                    fs::remove_file(&index_js)?;
                    cleaned.push("Node.js index.js".to_string());
                }

                if index_d_ts.exists() {
                    fs::remove_file(&index_d_ts)?;
                    cleaned.push("Node.js index.d.ts".to_string());
                }

                // Clean napi build artifacts
                let napi_dir = nodejs_dir.join("napi");
                if napi_dir.exists() {
                    fs::remove_dir_all(&napi_dir)?;
                    cleaned.push(format!("Napi build artifacts ({})", napi_dir.display()));
                }
            }
            "rust" => {
                // Clean cargo target directory (but keep it if it has other projects)
                let target_dir = project_root.join("target");
                if target_dir.exists() {
                    // Only clean if it's a BridgeRust-specific target
                    // For now, we'll clean the whole target directory
                    // Users can use `cargo clean` for more control
                    println!(
                        "  {} Note: Use 'cargo clean' to clean Rust build artifacts",
                        style("ℹ").yellow()
                    );
                }
            }
            _ => {}
        }
    }

    // Clean cache if requested
    if cache {
        let cache_dir = get_cache_dir()?;
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir)?;
            cleaned.push(format!("Build cache ({})", cache_dir.display()));
        }
    }

    if cleaned.is_empty() {
        println!("  {} Nothing to clean", style("ℹ").yellow());
    } else {
        println!("\n  {} Cleaned:", style("✓").green());
        for item in &cleaned {
            println!("    - {}", item);
        }
    }

    println!("\n{}", style("✅ Clean completed!").bold().green());
    Ok(())
}
