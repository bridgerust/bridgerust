use anyhow::{Context, Result};
use console::style;
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
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

pub async fn handle(target: String, dry_run: bool) -> Result<()> {
    let project_root = find_project_root()?;
    println!("  Project root: {}", project_root.display());

    if dry_run {
        println!(
            "{}",
            style("🔍 Dry run mode - no packages will be published").yellow()
        );
    } else {
        let confirm = Confirm::new()
            .with_prompt("Are you sure you want to publish? This will upload to PyPI/npm")
            .default(false)
            .interact()?;

        if !confirm {
            println!("{}", style("Cancelled.").yellow());
            return Ok(());
        }
    }

    println!("{}", style("📦 Publishing packages...").bold().cyan());

    let targets: Vec<&str> = match target.as_str() {
        "all" => vec!["python", "nodejs"],
        "python" => vec!["python"],
        "nodejs" => vec!["nodejs"],
        _ => anyhow::bail!(
            "Invalid target: {}. Use 'python', 'nodejs', or 'all'",
            target
        ),
    };

    // Pre-publish checks
    println!("  {} Running pre-publish checks...", style("→").cyan());
    for target in &targets {
        validate_publish_target(target, &project_root)?;
    }
    println!("  {} Pre-publish checks passed", style("✓").green());

    // Publish targets
    let mut failed = Vec::new();
    for target in &targets {
        match publish_target(target, dry_run, &project_root).await {
            Ok(_) => {}
            Err(e) => {
                failed.push((*target, e));
            }
        }
    }

    if !failed.is_empty() {
        eprintln!("\n{}", style("❌ Some publishes failed:").bold().red());
        for (target, error) in &failed {
            eprintln!("  {} {}: {}", style("✗").red(), target, error);
        }
        anyhow::bail!("Publishing failed");
    }

    println!("\n{}", style("✅ Publishing completed!").bold().green());
    Ok(())
}

fn validate_publish_target(target: &str, project_root: &Path) -> Result<()> {
    match target {
        "python" => {
            let python_dir = project_root.join("python");
            let pyproject_toml = python_dir.join("pyproject.toml");

            if !pyproject_toml.exists() {
                anyhow::bail!("Python package not configured. Run 'bridge init' or create python/pyproject.toml");
            }

            // Check if maturin is available
            let maturin_check = Command::new("maturin").arg("--version").output();

            if maturin_check.is_err() {
                anyhow::bail!("maturin is not installed. Install it with: pip install maturin");
            }
        }
        "nodejs" => {
            let nodejs_dir = project_root.join("nodejs");
            let package_json = nodejs_dir.join("package.json");

            if !package_json.exists() {
                anyhow::bail!("Node.js package not configured. Run 'bridge init' or create nodejs/package.json");
            }

            // Check if npm is available
            let npm_check = Command::new("npm").arg("--version").output();

            if npm_check.is_err() {
                anyhow::bail!("npm is not installed or not in PATH");
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

async fn publish_target(target: &str, dry_run: bool, project_root: &Path) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Publishing {} package...", target));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    match target {
        "python" => {
            let python_dir = project_root.join("python");

            pb.finish_and_clear();
            let mut cmd = Command::new("maturin");
            cmd.current_dir(&python_dir);
            cmd.arg("publish");
            if dry_run {
                cmd.arg("--dry-run");
            }
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());

            let status = cmd.status().context("Failed to run maturin publish")?;

            if !status.success() {
                anyhow::bail!("Python publish failed");
            }
            println!("  {} Python package published", style("✓").green());
        }
        "nodejs" => {
            let nodejs_dir = project_root.join("nodejs");

            pb.finish_and_clear();
            let mut cmd = Command::new("npm");
            cmd.current_dir(&nodejs_dir);
            cmd.arg("publish");
            if dry_run {
                cmd.arg("--dry-run");
            }
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());

            let status = cmd.status().context("Failed to run npm publish")?;

            if !status.success() {
                anyhow::bail!("Node.js publish failed");
            }
            println!("  {} Node.js package published", style("✓").green());
        }
        _ => unreachable!(),
    }

    Ok(())
}
