use anyhow::{Context, Result};
use console::style;
use std::process::Command;

pub async fn handle(target: String, open: bool) -> Result<()> {
    println!(
        "{}",
        style(format!("📚 Generating documentation (target: {})", target))
            .bold()
            .cyan()
    );

    // Always generate Rust docs as base
    println!("\n{}", style("🦀 Generating Rust docs...").cyan());
    let mut cargo_doc = Command::new("cargo");
    cargo_doc.arg("doc");
    cargo_doc.arg("--no-deps"); // Focus on current crate

    if open && target == "rust" {
        cargo_doc.arg("--open");
    }

    let status = cargo_doc.status().context("Failed to run cargo doc")?;
    if !status.success() {
        anyhow::bail!("Failed to generate Rust docs");
    }

    if target == "python" || target == "all" {
        println!("\n{}", style("🐍 Generating Python docs...").cyan());
        // Only if pdoc is installed
        if Command::new("pdoc").arg("--version").output().is_ok() {
            // Assuming the package is installed in the current venv
            // We need to know the package name.
            // Ideally we'd parse pyproject.toml, but for now let's just warn.
            println!("  Run `pdoc <package_name>` to view Python docs.");
            if open {
                println!("  (Skipping auto-open for Python docs)");
            }
        } else {
            println!(
                "  {} pdoc not found. Install it with `pip install pdoc` for better Python docs.",
                style("⚠").yellow()
            );
        }
    }

    if target == "nodejs" || target == "all" {
        println!(
            "\n{}",
            style("Node.js docs generation not yet fully supported.").yellow()
        );
        println!("  (Type definitions .d.ts are generated during build)");
    }

    println!("\n{}", style("√ Docs generation complete!").green());
    Ok(())
}
