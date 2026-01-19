use anyhow::{Context, Result};
use console::style;
use std::fs;
use std::path::Path;
use std::process::Command;

pub async fn handle(name: String, _template: String, yes: bool) -> Result<()> {
    println!(
        "{}",
        style(format!("🚀 Creating new BridgeRust project: {}", name))
            .bold()
            .cyan()
    );

    let path = Path::new(&name);
    if path.exists() {
        if !yes {
            anyhow::bail!("Directory '{}' already exists", name);
        }
        println!(
            "  {} Directory exists (overwriting...)",
            style("⚠").yellow()
        );
    } else {
        fs::create_dir_all(path).context("Failed to create project directory")?;
    }

    // initialize git
    if Command::new("git")
        .arg("init")
        .current_dir(path)
        .output()
        .is_ok()
    {
        println!("  {} Initialized git repository", style("✓").green());
    }

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
bridgerust = {{ version = "0.1", features = ["python", "nodejs"] }}
"#,
        name
    );
    fs::write(path.join("Cargo.toml"), cargo_toml)?;
    println!("  {} Created Cargo.toml", style("✓").green());

    // Create src directory and lib.rs
    fs::create_dir_all(path.join("src"))?;
    let lib_rs = r#"use bridgerust::bridge;

#[bridge]
pub fn hello() -> String {
    "Hello from BridgeRust!".to_string()
}
"#;
    fs::write(path.join("src/lib.rs"), lib_rs)?;
    println!("  {} Created src/lib.rs", style("✓").green());

    // Create pyproject.toml
    fs::create_dir_all(path.join("python"))?;
    let pyproject_toml = format!(
        r#"[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "{}"
requires-python = ">=3.8"
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
    "Programming Language :: Python :: Implementation :: PyPy",
]
"#,
        name.replace("-", "_")
    );
    fs::write(path.join("python/pyproject.toml"), pyproject_toml)?;
    println!("  {} Created python/pyproject.toml", style("✓").green());

    // Create package.json
    fs::create_dir_all(path.join("nodejs"))?;
    let package_json = format!(
        r#"{{
  "name": "{}",
  "version": "0.1.0",
  "main": "index.js",
  "devDependencies": {{
    "@napi-rs/cli": "^2.0.0"
  }},
  "napi": {{
    "name": "{}"
  }}
}}
"#,
        name,
        name.replace("-", "_")
    );
    fs::write(path.join("nodejs/package.json"), package_json)?;
    println!("  {} Created nodejs/package.json", style("✓").green());

    // Create .gitignore
    let gitignore = r#"/target
/dist
/node_modules
__pycache__
*.pyc
*.whl
*.tgz
"#;
    fs::write(path.join(".gitignore"), gitignore)?;
    println!("  {} Created .gitignore", style("✓").green());

    println!(
        "\n{}",
        style("√ Project created successfully!").bold().green()
    );
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  bridge build --all");

    Ok(())
}
