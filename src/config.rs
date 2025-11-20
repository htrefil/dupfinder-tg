use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::{Config, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Path to the configuration file
    #[arg(short, long, global = true, default_value = "config.toml")]
    pub config: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

// The enum for your subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
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

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelegramSettings {
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub telegram: TelegramSettings,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: u8,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_similarity_threshold() -> u8 {
    5
}

impl Settings {
    pub fn new(args: &Cli) -> Result<Self> {
        let builder = Config::builder()
            // required(false) so it doesn't crash if you only use Env Vars
            .add_source(File::from(args.config.as_path()).required(false))
            .add_source(Environment::with_prefix("DUPFINDER").separator("_"));

        let config = builder.build().context("Failed to build configuration")?;

        config
            .try_deserialize()
            .context("Failed to deserialize configuration")
    }
}
