use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelegramSettings {
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub database: DatabaseSettings,
    pub telegram: TelegramSettings,
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: u8,
}

fn default_similarity_threshold() -> u8 {
    5
}
