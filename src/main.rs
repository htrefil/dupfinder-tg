mod bot;
mod config;
mod database;
mod importer;

use anyhow::Result;
use clap::Parser;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli = config::Cli::parse();
    let settings = config::Settings::new(&cli)?;

    tracing_subscriber::fmt()
        .with_env_filter(&settings.log_level)
        .init();

    info!("Configuration loaded. Connecting to database...");

    let pool = database::init_pool(&settings.database.url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    info!("Database connected.");

    match cli.command {
        config::Commands::Run => {
            info!("Starting bot...");
            bot::run(settings, pool).await?;
        }
        config::Commands::Import { path, chat_id } => {
            info!("Running importer...");
            importer::run(&pool, &path, chat_id).await?;
        }
    }

    Ok(())
}
