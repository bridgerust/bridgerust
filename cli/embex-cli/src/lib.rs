use clap::{Parser, Subcommand};
use env_logger::Env;

pub mod commands;
pub mod declarative;

#[derive(Parser)]
#[command(name = "embex")]
#[command(about = "Embex Vector Database CLI Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage database migrations
    Migrate {
        #[command(subcommand)]
        action: MigrateAction,
    },
    /// Generate new resources
    Generate {
        /// Type of resource to generate (e.g., migration)
        resource: String,
        /// Name of the resource
        name: String,
    },
}

#[derive(Subcommand, Clone)]
pub enum MigrateAction {
    /// Run pending migrations
    Up,
    /// Rollback the last migration
    Down,
    /// Show migration status
    Status,
}

pub async fn run(args: Vec<String>) -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    dotenvy::dotenv().ok();

    // Parse from args, skipping the first argument if it's the program name (which is standard)
    // However, since we are calling this from bindings, we might get just the args without prog name.
    // Let's assume standard behavior: prog name + args.
    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Migrate { action } => {
            commands::migrate::handle(action).await?;
        }
        Commands::Generate { resource, name } => {
            commands::generate::handle(resource, name).await?;
        }
    }

    Ok(())
}
