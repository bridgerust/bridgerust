use anyhow::{Context, Result};
use console::style;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub async fn handle(verbose: bool) -> Result<()> {
    println!(
        "{}",
        style("🔍 Checking BridgeRust project...\n").bold().cyan()
    );

    let project_root = find_project_root()?;
    if verbose {
        println!("  Project root: {}", project_root.display());
    }

    let mut passed = 0;
    let mut failed = 0;

    // Define checks
    let checks = vec![
        (
            "Project Structure",
            check_project_structure(&project_root, verbose),
        ),
        ("Configuration", check_config(&project_root, verbose)),
        ("Cargo.toml", check_cargo_toml(&project_root, verbose)),
        ("Source Code", check_source_code(&project_root, verbose)),
        ("Python Setup", check_python_setup(&project_root, verbose)),
        ("Node.js Setup", check_nodejs_setup(&project_root, verbose)),
        ("Compilation", check_compilation(&project_root, verbose)),
    ];

    for (name, result) in checks {
        match result {
            Ok(()) => {
                // Individual checks print their own success/info messages
                passed += 1;
            }
            Err(e) => {
                println!("❌ {} - {}", style(name).bold(), e);
                failed += 1;
            }
        }
    }

    println!(
        "\n{}",
        style(format!("📊 Results: {} passed, {} failed", passed, failed)).bold()
    );

    if failed > 0 {
        println!(
            "\n{}",
            style("💡 Run 'bridge check --verbose' for details").yellow()
        );
        std::process::exit(1);
    }

    println!(
        "\n{}",
        style("✨ Project is ready for building!").bold().green()
    );
    Ok(())
}

fn find_project_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;

    loop {
        if current.join("Cargo.toml").exists() || current.join("bridgerust.toml").exists() {
            return Ok(current);
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => anyhow::bail!("Not in a BridgeRust project. Run 'bridge init' to create one."),
        }
    }
}

fn check_project_structure(root: &Path, verbose: bool) -> Result<()> {
    println!("\n{} Checking project structure...", style("📁").cyan());

    let required = vec![
        ("Cargo.toml", "Rust project configuration"),
        ("src/lib.rs", "Main library source file"),
    ];

    let mut missing = Vec::new();
    for (file, desc) in required {
        let path = root.join(file);
        if !path.exists() {
            missing.push((file, desc));
        } else if verbose {
            println!("  {} {} ({})", style("✓").green(), file, desc);
        } else {
            println!("  {} {}", style("✓").green(), file);
        }
    }

    if !missing.is_empty() {
        eprintln!("  {} Missing required files:", style("✗").red());
        for (file, desc) in missing {
            eprintln!("    - {} ({})", file, desc);
        }
        anyhow::bail!("Project structure is incomplete");
    }

    Ok(())
}

fn check_config(root: &Path, verbose: bool) -> Result<()> {
    println!("\n{} Checking configuration...", style("⚙️").cyan());

    let config_path = root.join("bridgerust.toml");
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        if content.trim().is_empty() {
            eprintln!("  {} bridgerust.toml is empty", style("⚠").yellow());
        } else {
            println!("  {} bridgerust.toml found", style("✓").green());

            // Validate package names if verbose
            if verbose {
                // Simple validation without toml parsing
                if content.contains("module_name") {
                    println!("    {} Python module name configured", style("✓").green());
                }
                if content.contains("package_name") {
                    println!("    {} Node.js package name configured", style("✓").green());
                }
            }
        }
    } else {
        eprintln!(
            "  {} bridgerust.toml not found (optional)",
            style("⚠").yellow()
        );
    }

    Ok(())
}

fn check_cargo_toml(root: &Path, _verbose: bool) -> Result<()> {
    println!("\n{} Checking Cargo.toml...", style("📦").cyan());

    let cargo_toml = root.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml).context("Failed to read Cargo.toml")?;

    // Check for bridgerust dependency
    if content.contains("bridgerust") {
        println!("  {} bridgerust dependency found", style("✓").green());
    } else {
        eprintln!("  {} bridgerust dependency not found", style("✗").red());
        anyhow::bail!("Cargo.toml missing bridgerust dependency");
    }

    // Check for features
    let has_python =
        content.contains("feature = \"python\"") || content.contains("features = [\"python\"]");
    let has_nodejs =
        content.contains("feature = \"nodejs\"") || content.contains("features = [\"nodejs\"]");

    if has_python {
        println!("  {} Python feature enabled", style("✓").green());
    } else {
        eprintln!("  {} Python feature not enabled", style("⚠").yellow());
    }

    if has_nodejs {
        println!("  {} Node.js feature enabled", style("✓").green());
    } else {
        eprintln!("  {} Node.js feature not enabled", style("⚠").yellow());
    }

    if !has_python && !has_nodejs {
        eprintln!("  {} No target features enabled", style("⚠").yellow());
    }

    Ok(())
}

fn check_source_code(root: &Path, verbose: bool) -> Result<()> {
    println!("\n{} Checking source code...", style("📝").cyan());

    let lib_rs = root.join("src/lib.rs");
    if !lib_rs.exists() {
        anyhow::bail!("src/lib.rs not found");
    }

    let content = fs::read_to_string(&lib_rs)?;

    // Check for bridgerust imports
    if content.contains("bridgerust") {
        println!("  {} bridgerust imports found", style("✓").green());
    } else {
        eprintln!("  {} No bridgerust imports found", style("⚠").yellow());
    }

    // Check for #[export] usage
    let export_count =
        content.matches("#[export]").count() + content.matches("#[bridgerust::export]").count();
    if export_count > 0 {
        println!(
            "  {} #[export] macros found ({})",
            style("✓").green(),
            export_count
        );

        if verbose {
            // Check for common issues
            let lines: Vec<&str> = content.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if line.contains("#[bridgerust::export]") || line.contains("#[export]") {
                    // Check if next non-empty line has pub
                    let mut found_pub = false;
                    for next_line_raw in lines.iter().skip(i + 1) {
                        let next_line = next_line_raw.trim();
                        if next_line.is_empty() || next_line.starts_with("//") {
                            continue;
                        }
                        if next_line.contains("pub fn")
                            || next_line.contains("pub struct")
                            || next_line.contains("pub enum")
                        {
                            found_pub = true;
                        }
                        break;
                    }
                    if !found_pub && verbose {
                        eprintln!(
                            "    {} Line {}: #[export] may not be on a public item",
                            style("⚠").yellow(),
                            i + 1
                        );
                    }
                }
            }
        }
    } else {
        eprintln!("  {} No #[export] macros found", style("⚠").yellow());
        eprintln!("    Add #[bridgerust::export] to functions/structs you want to export");
    }

    // Check for #[error] usage
    let error_count =
        content.matches("#[error]").count() + content.matches("#[bridgerust::error]").count();
    if error_count > 0 {
        println!(
            "  {} #[error] macros found ({})",
            style("✓").green(),
            error_count
        );
    }

    Ok(())
}

fn check_python_setup(root: &Path, _verbose: bool) -> Result<()> {
    println!("\n{} Checking Python setup...", style("🐍").cyan());

    let python_dir = root.join("python");
    let pyproject = python_dir.join("pyproject.toml");

    if pyproject.exists() {
        println!("  {} Python project found", style("✓").green());

        // Check if maturin is available
        let output = Command::new("maturin").arg("--version").output();

        match output {
            Ok(_) => println!("  {} maturin is installed", style("✓").green()),
            Err(_) => eprintln!(
                "  {} maturin not found (install with: pip install maturin)",
                style("⚠").yellow()
            ),
        }
    } else {
        eprintln!(
            "  {} Python project not found (optional)",
            style("⚠").yellow()
        );
    }

    Ok(())
}

fn check_nodejs_setup(root: &Path, _verbose: bool) -> Result<()> {
    println!("\n{} Checking Node.js setup...", style("📗").cyan());

    let nodejs_dir = root.join("nodejs");
    let package_json = nodejs_dir.join("package.json");

    if package_json.exists() {
        println!("  {} Node.js project found", style("✓").green());

        // Check if npm is available
        let output = Command::new("npm").arg("--version").output();

        match output {
            Ok(_) => println!("  {} npm is installed", style("✓").green()),
            Err(_) => eprintln!("  {} npm not found", style("⚠").yellow()),
        }

        // Check if napi-rs CLI is available (via npx)
        let output = Command::new("npx")
            .arg("@napi-rs/cli")
            .arg("--version")
            .output();

        match output {
            Ok(_) => println!("  {} @napi-rs/cli available via npx", style("✓").green()),
            Err(_) => eprintln!(
                "  {} @napi-rs/cli not available (will be installed on build)",
                style("⚠").yellow()
            ),
        }
    } else {
        eprintln!(
            "  {} Node.js project not found (optional)",
            style("⚠").yellow()
        );
    }

    Ok(())
}

fn check_compilation(root: &Path, verbose: bool) -> Result<()> {
    println!("\n{} Checking compilation...", style("🔨").cyan());

    // Check Rust compilation (cargo check)
    let mut cmd = Command::new("cargo");
    cmd.arg("check");
    cmd.current_dir(root);

    if verbose {
        println!("  Running cargo check...");
    }

    let output = if verbose {
        cmd.stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()
    } else {
        cmd.output()
    }
    .context("Failed to run cargo check")?;

    if output.status.success() {
        println!("  {} Code compiles successfully", style("✓").green());
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("  {} Compilation errors found:", style("✗").red());
        if !verbose {
            eprintln!("{}", stderr);
        }
        anyhow::bail!("Code does not compile. Fix errors before building.");
    }

    Ok(())
}
