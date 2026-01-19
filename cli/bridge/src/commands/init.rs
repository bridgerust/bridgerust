use anyhow::{Context, Result};
use console::style;
use dialoguer::{Confirm, Input};
use std::fs;
use std::path::PathBuf;

use crate::templates;

pub async fn handle(name: String, skip_prompts: bool) -> Result<()> {
    println!(
        "{}",
        style("🚀 Initializing BridgeRust project...").bold().cyan()
    );

    let project_dir = PathBuf::from(&name);

    if project_dir.exists() {
        if !skip_prompts {
            let overwrite = Confirm::new()
                .with_prompt(format!("Directory '{}' already exists. Overwrite?", name))
                .default(false)
                .interact()?;

            if !overwrite {
                println!("{}", style("Cancelled.").yellow());
                return Ok(());
            }
        }
        fs::remove_dir_all(&project_dir)?;
    }

    fs::create_dir_all(&project_dir)?;

    // Get project metadata
    let description = if skip_prompts {
        format!("A BridgeRust project: {}", name)
    } else {
        Input::new()
            .with_prompt("Project description")
            .default(format!("A BridgeRust project: {}", name))
            .interact()?
    };

    let author = if skip_prompts {
        "BridgeRust Team".to_string()
    } else {
        Input::new()
            .with_prompt("Author")
            .default("BridgeRust Team".to_string())
            .interact()?
    };

    // Generate project structure
    templates::generate_project(&project_dir, &name, &description, &author)
        .context("Failed to generate project template")?;

    println!(
        "\n{}",
        style("√ Project created successfully!").bold().green()
    );
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  bridge build --all");
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
