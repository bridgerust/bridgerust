use anyhow::{Context, Result};
use console::style;
use std::fs;
use std::path::PathBuf;

use crate::templates;

pub async fn handle(output: String) -> Result<()> {
    println!(
        "{}",
        style("📋 Generating CI/CD workflows...").bold().cyan()
    );

    let output_dir = PathBuf::from(&output);
    fs::create_dir_all(&output_dir).context(format!("Failed to create directory: {}", output))?;

    templates::generate_workflows(&output_dir).context("Failed to generate workflow files")?;

    println!("  {} CI workflow (ci.yml) generated", style("✓").green());
    println!(
        "  {} Release workflow (release.yml) generated",
        style("✓").green()
    );
    println!("  {} Workflows generated in {}", style("✓").green(), output);

    println!("\n{}", style("📝 Next steps:").bold().cyan());
    println!("  1. Review the generated workflow files");
    println!("  2. Add secrets to your GitHub repository:");
    println!("     - PYPI_API_TOKEN (for Python publishing)");
    println!("     - NPM_TOKEN (for Node.js publishing)");
    println!("  3. Create a 'publish' environment in GitHub (optional, for manual approval)");

    println!(
        "\n{}",
        style("✅ Workflow generation completed!").bold().green()
    );
    Ok(())
}
