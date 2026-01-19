use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
pub mod config;
pub mod templates;

use commands::*;

#[derive(Parser)]
#[command(name = "bridge")]
#[command(about = "BridgeRust CLI - Write Rust once, deploy to Python and Node.js", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start development server (watch & rebuild)
    Dev {
        /// Dev target: python, nodejs, or all
        #[arg(short, long, default_value = "all")]
        target: String,

        /// Port for reload server (placeholder)
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Generate documentation
    Docs {
        /// Generate docs for target: rust, python, nodejs, or all
        #[arg(short, long, default_value = "all")]
        target: String,

        /// Open docs in browser
        #[arg(short, long)]
        open: bool,
    },
    /// Run project benchmarks
    Benchmark {
        /// Arguments passed to benchmarks
        #[arg(allow_hyphen_values = true)]
        params: Option<String>,
    },
    /// Manage project templates
    Template {
        /// List available templates
        #[arg(short, long)]
        list: bool,
    },
    /// Create a new BridgeRust project from a template
    New {
        /// Project name
        name: String,

        /// Template to use
        #[arg(short, long, default_value = "basic")]
        template: String,

        /// Skip interactive prompts
        #[arg(short, long)]
        yes: bool,
    },
    /// Initialize a new BridgeRust project
    Init {
        /// Project name
        name: String,
        /// Skip interactive prompts
        #[arg(short, long)]
        yes: bool,
    },
    /// Integrate BridgeRust into an existing Rust project
    Integrate {
        /// Skip interactive prompts
        #[arg(short, long)]
        yes: bool,
        /// Add example code to src/lib.rs
        #[arg(short, long)]
        example: bool,
    },
    /// Build Python and/or Node.js bindings
    Build {
        /// Build target: python, nodejs, or all
        #[arg(short, long, default_value = "all")]
        target: String,
        /// Release mode
        #[arg(short, long)]
        release: bool,
    },
    /// Run tests for Python and/or Node.js bindings
    Test {
        /// Test target: python, nodejs, or all
        #[arg(short, long, default_value = "all")]
        target: String,
    },
    /// Publish packages to PyPI and/or npm
    Publish {
        /// Publish target: python, nodejs, or all
        #[arg(short, long, default_value = "all")]
        target: String,
        /// Dry run (don't actually publish)
        #[arg(long)]
        dry_run: bool,
    },
    /// Generate CI/CD workflow files
    Workflows {
        /// Output directory for workflow files
        #[arg(short, long, default_value = ".github/workflows")]
        output: String,
    },
    /// Validate project structure and configuration
    Check {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Clean build artifacts and cache
    Clean {
        /// Clean target: python, nodejs, rust, or all
        #[arg(short, long)]
        target: Option<String>,
        /// Also clean build cache
        #[arg(short, long)]
        cache: bool,
    },
    /// Watch for file changes and rebuild automatically
    Watch {
        /// Watch target: python, nodejs, or all
        #[arg(short, long, default_value = "all")]
        target: String,
        /// Also run tests after each build
        #[arg(short, long)]
        test: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dev { target, port } => dev::handle(target, port).await,
        Commands::Docs { target, open } => docs::handle(target, open).await,
        Commands::Benchmark { params } => benchmark::handle(params).await,
        Commands::Template { list } => template::handle(list).await,
        Commands::New {
            name,
            template,
            yes,
        } => new::handle(name, template, yes).await,
        Commands::Init { name, yes } => init::handle(name, yes).await,
        Commands::Integrate { yes, example } => integrate::handle(yes, example).await,
        Commands::Build { target, release } => build::handle(target, release).await,
        Commands::Test { target } => test::handle(target).await,
        Commands::Publish { target, dry_run } => publish::handle(target, dry_run).await,
        Commands::Workflows { output } => workflows::handle(output).await,
        Commands::Check { verbose } => check::handle(verbose).await,
        Commands::Clean { target, cache } => clean::handle(target, cache).await,
        Commands::Watch { target, test } => watch::handle(target, test).await,
    }
}
