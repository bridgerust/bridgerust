use anyhow::{Context, Result};
use console::style;
use dialoguer::Confirm;
use std::fs;
use std::path::PathBuf;
use toml::Value;

/// Find the project root by looking for Cargo.toml
fn find_project_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(current);
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => anyhow::bail!(
                "Not in a Rust project. Run this command from a directory with Cargo.toml"
            ),
        }
    }
}

pub async fn handle(skip_prompts: bool, add_example: bool) -> Result<()> {
    println!(
        "{}",
        style("🔧 Integrating BridgeRust into existing project...")
            .bold()
            .cyan()
    );

    let project_root = find_project_root()?;
    println!("  Project root: {}", project_root.display());

    let cargo_toml_path = project_root.join("Cargo.toml");
    let cargo_content =
        fs::read_to_string(&cargo_toml_path).context("Failed to read Cargo.toml")?;

    // Parse existing Cargo.toml
    let mut cargo_toml: Value =
        toml::from_str(&cargo_content).context("Failed to parse Cargo.toml")?;

    // Get project metadata (before mutating cargo_toml)
    let project_name = cargo_toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .context("Cargo.toml missing package.name")?
        .to_string();

    let version = cargo_toml
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("0.1.0")
        .to_string();

    let description = cargo_toml
        .get("package")
        .and_then(|p| p.get("description"))
        .and_then(|d| d.as_str())
        .unwrap_or("A Rust project with BridgeRust bindings")
        .to_string();

    println!("  Project: {} (v{})", project_name, version);

    // Check if already integrated
    if cargo_content.contains("bridgerust") && !skip_prompts {
        let overwrite = Confirm::new()
            .with_prompt("BridgeRust appears to already be integrated. Continue anyway?")
            .default(false)
            .interact()?;

        if !overwrite {
            println!("{}", style("Cancelled.").yellow());
            return Ok(());
        }
    }

    // Update Cargo.toml
    println!("\n{} Updating Cargo.toml...", style("📦").cyan());
    update_cargo_toml(&mut cargo_toml)?;

    // Write updated Cargo.toml
    let updated_content =
        toml::to_string_pretty(&cargo_toml).context("Failed to serialize Cargo.toml")?;
    fs::write(&cargo_toml_path, updated_content)?;
    println!("  {} Cargo.toml updated", style("✓").green());

    // Create directory structure
    println!("\n{} Creating directory structure...", style("📁").cyan());
    fs::create_dir_all(project_root.join("python"))?;
    fs::create_dir_all(project_root.join("nodejs"))?;
    println!("  {} python/ directory", style("✓").green());
    println!("  {} nodejs/ directory", style("✓").green());

    // Generate bridgerust.toml
    println!("\n{} Creating configuration files...", style("⚙️").cyan());
    let config_toml = format!(
        r#"[package]
name = "{}"
version = "{}"
description = "{}"

[python]
module_name = "{}"

[nodejs]
package_name = "@bridgerust/{}"
"#,
        project_name, version, description, project_name, project_name
    );

    let config_path = project_root.join("bridgerust.toml");
    if config_path.exists() && !skip_prompts {
        let overwrite = Confirm::new()
            .with_prompt("bridgerust.toml already exists. Overwrite?")
            .default(false)
            .interact()?;
        if !overwrite {
            println!("  {} Skipping bridgerust.toml", style("⚠").yellow());
        } else {
            fs::write(&config_path, config_toml)?;
            println!("  {} bridgerust.toml created", style("✓").green());
        }
    } else {
        fs::write(&config_path, config_toml)?;
        println!("  {} bridgerust.toml created", style("✓").green());
    }

    // Generate pyproject.toml
    let pyproject_path = project_root.join("python/pyproject.toml");
    if !pyproject_path.exists() {
        let pyproject_toml = format!(
            r#"[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "{}"
version = "{}"
description = "{}"
requires-python = ">=3.8"
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
    "Programming Language :: Python :: Implementation :: PyPy",
]

[tool.maturin]
module-name = "{}"
"#,
            project_name, version, description, project_name
        );
        fs::write(&pyproject_path, pyproject_toml)?;
        println!("  {} python/pyproject.toml created", style("✓").green());
    } else {
        println!(
            "  {} python/pyproject.toml already exists",
            style("⚠").yellow()
        );
    }

    // Generate package.json
    let package_json_path = project_root.join("nodejs/package.json");
    if !package_json_path.exists() {
        let package_json = format!(
            r#"{{
  "name": "@bridgerust/{}",
  "version": "{}",
  "description": "{}",
  "main": "index.js",
  "types": "index.d.ts",
  "files": [
    "index.js",
    "index.d.ts",
    "*.node"
  ],
  "napi": {{
    "name": "{}",
    "triples": {{
      "defaults": true,
      "additional": []
    }}
  }},
  "scripts": {{
    "build": "npx --yes @napi-rs/cli build --platform"
  }}
}}
"#,
            project_name, version, description, project_name
        );
        fs::write(&package_json_path, package_json)?;
        println!("  {} nodejs/package.json created", style("✓").green());
    } else {
        println!(
            "  {} nodejs/package.json already exists",
            style("⚠").yellow()
        );
    }

    // Optionally add example code to lib.rs
    if add_example {
        println!("\n{} Adding example code...", style("📝").cyan());
        let lib_rs_path = project_root.join("src/lib.rs");
        if lib_rs_path.exists() {
            let mut lib_content = fs::read_to_string(&lib_rs_path)?;

            if !lib_content.contains("bridgerust") {
                lib_content = format!("use bridgerust::export;\n\n{}", lib_content);

                let example = r#"
// Example BridgeRust export - remove this and add your own #[bridgerust::export] functions
#[bridgerust::export]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
"#;
                lib_content.push_str(example);

                fs::write(&lib_rs_path, lib_content)?;
                println!("  {} Example code added to src/lib.rs", style("✓").green());
            } else {
                println!(
                    "  {} src/lib.rs already uses bridgerust",
                    style("⚠").yellow()
                );
            }
        } else {
            println!(
                "  {} src/lib.rs not found, skipping example code",
                style("⚠").yellow()
            );
        }
    }

    println!(
        "\n{}",
        style("✅ BridgeRust integration complete!").bold().green()
    );
    println!("\n{} Next steps:", style("📝").bold().cyan());
    println!("  1. Add #[bridgerust::export] to functions/structs you want to export");
    println!("  2. Run: bridge build --all");
    println!("  3. Run: bridge test --all");
    println!("\n{}", style("Happy coding! 🎉").bold().cyan());

    // GitHub star CTA
    println!("\n{}", style("💡 Enjoying BridgeRust?").bold().yellow());
    println!("  If BridgeRust helps you build faster, please consider giving us a star!");
    println!(
        "  {}",
        style("⭐ https://github.com/bridgerust/bridgerust").cyan()
    );
    println!("  Your support helps others discover the project! 🙏");

    Ok(())
}

fn update_cargo_toml(cargo_toml: &mut Value) -> Result<()> {
    if cargo_toml.get("lib").is_none() {
        let mut lib = toml::map::Map::new();
        lib.insert(
            "crate-type".to_string(),
            Value::Array(vec![
                Value::String("cdylib".to_string()),
                Value::String("rlib".to_string()),
            ]),
        );
        cargo_toml
            .as_table_mut()
            .context("Cargo.toml is not a table")?
            .insert("lib".to_string(), Value::Table(lib));
        println!("  {} Added [lib] section", style("✓").green());
    }

    let deps = cargo_toml
        .get_mut("dependencies")
        .and_then(|d| d.as_table_mut())
        .context("Failed to get dependencies section")?;

    if !deps.contains_key("bridgerust") {
        let mut bridgerust = toml::map::Map::new();
        bridgerust.insert(
            "path".to_string(),
            Value::String("../../crates/bridgerust".to_string()),
        );
        bridgerust.insert("version".to_string(), Value::String("0.1".to_string()));
        deps.insert("bridgerust".to_string(), Value::Table(bridgerust));
        println!("  {} Added bridgerust dependency", style("✓").green());
    } else {
        println!(
            "  {} bridgerust dependency already exists",
            style("⚠").yellow()
        );
    }

    if !deps.contains_key("pyo3") {
        let mut pyo3 = toml::map::Map::new();
        pyo3.insert("version".to_string(), Value::String("0.27".to_string()));
        pyo3.insert("optional".to_string(), Value::Boolean(true));
        let mut features = toml::map::Map::new();
        features.insert("extension-module".to_string(), Value::String(String::new()));
        pyo3.insert(
            "features".to_string(),
            Value::Array(vec![Value::String("extension-module".to_string())]),
        );
        deps.insert("pyo3".to_string(), Value::Table(pyo3));
        println!("  {} Added pyo3 dependency", style("✓").green());
    }

    if !deps.contains_key("napi") {
        let mut napi = toml::map::Map::new();
        napi.insert("version".to_string(), Value::String("3".to_string()));
        napi.insert("optional".to_string(), Value::Boolean(true));
        deps.insert("napi".to_string(), Value::Table(napi));
        println!("  {} Added napi dependency", style("✓").green());
    }

    if !deps.contains_key("napi-derive") {
        let mut napi_derive = toml::map::Map::new();
        napi_derive.insert("version".to_string(), Value::String("3".to_string()));
        napi_derive.insert("optional".to_string(), Value::Boolean(true));
        deps.insert("napi-derive".to_string(), Value::Table(napi_derive));
        println!("  {} Added napi-derive dependency", style("✓").green());
    }

    let features = if let Some(f) = cargo_toml
        .get_mut("features")
        .and_then(|f| f.as_table_mut())
    {
        f
    } else {
        let features_table = toml::map::Map::new();
        cargo_toml
            .as_table_mut()
            .context("Cargo.toml is not a table")?
            .insert("features".to_string(), Value::Table(features_table));
        cargo_toml
            .get_mut("features")
            .and_then(|f| f.as_table_mut())
            .context("Failed to create features section")?
    };

    if !features.contains_key("python") {
        features.insert(
            "python".to_string(),
            Value::Array(vec![
                Value::String("dep:pyo3".to_string()),
                Value::String("bridgerust/python".to_string()),
            ]),
        );
        println!("  {} Added python feature", style("✓").green());
    }

    if !features.contains_key("nodejs") {
        features.insert(
            "nodejs".to_string(),
            Value::Array(vec![
                Value::String("dep:napi".to_string()),
                Value::String("dep:napi-derive".to_string()),
                Value::String("bridgerust/nodejs".to_string()),
            ]),
        );
        println!("  {} Added nodejs feature", style("✓").green());
    }

    Ok(())
}
