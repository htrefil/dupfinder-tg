mod bot;
mod config;
mod database;
mod importer;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::Config;
use std::path::PathBuf;
use tokio::fs;
use tracing::level_filters::LevelFilter;
use tracing::{info, subscriber};
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Path to the configuration file
    #[arg(short, long, global = true, default_value = "config.toml")]
    pub config: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

// The enum for your subcommands
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run the Telegram bot and listen for new messages
    Run,
    /// Import data from a Telegram JSON chat export
    Import {
        /// Path to the chat export's result.json file
        #[arg(required = true)]
        path: PathBuf,
        /// the BOT-FACING chat id (might be different from the one in the file)
        #[arg(required = true, allow_negative_numbers = true)]
        chat_id: i64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().without_time().with_target(false));

    subscriber::set_global_default(registry).unwrap();

    // Log tracing adapter for teloxide.
    LogTracer::init().unwrap();

    let cli = Cli::parse();

    let config = fs::read_to_string(&cli.config)
        .await
        .context("error reading config")?;

    let config = toml::from_str::<Config>(&config).context("error parsing config")?;

    info!("Configuration loaded. Connecting to database...");

    let pool = database::init_pool(&config.database.url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    info!("Database connected.");

    match cli.command {
        Command::Run => {
            info!("Starting bot...");
            bot::run(config, pool).await?;
        }
        Command::Import { path, chat_id } => {
            info!("Running importer...");
            importer::run(&pool, &path, chat_id).await?;
        }
    }

    Ok(())
}
