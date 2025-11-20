// src/importer.rs
use crate::database;
use anyhow::Result;
use img_hash::HasherConfig;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use sqlx::PgPool;
use std::fs::File;
use std::path::{Path, PathBuf};
use thiserror::Error;

// --- Structs to model the Telegram JSON export ---
#[derive(Deserialize, Debug)]
struct Export {
    name: String,
    messages: Vec<Message>,
}

#[derive(Deserialize, Debug)]
struct Message {
    id: i32,
    #[serde(rename = "type")]
    message_type: String,
    photo: Option<PathBuf>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error ({path})")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("couldnt parse json")]
    Json(#[from] serde_json::Error),
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

// The main function for the importer
pub async fn run(pool: &PgPool, path: &Path, chat_id: i64) -> Result<(), Error> {
    println!("▶️ Starting import from: {}", path.display());

    // --- 1. Parse the JSON file ---
    let file = File::open(path).map_err(|e| Error::Io {
        path: path.to_owned(),
        source: e,
    })?;
    let data: Export = serde_json::from_reader(file)?;
    let base_path = path.parent().unwrap();

    let chat_title = data.name;
    println!(
        "Chat: '{}' with {} messages.",
        chat_title,
        data.messages.len()
    );

    // --- 2. Setup Hasher and Progress Bar ---
    let hasher = HasherConfig::new().to_hasher();
    let pb = ProgressBar::new(data.messages.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})", // LLM slop
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    // --- 3. Loop through messages and process images ---
    for msg in data.messages {
        pb.inc(1);
        if msg.message_type != "message" {
            continue;
        }

        let image_path = match msg.photo {
            Some(p) => base_path.join(p),
            None => continue,
        };

        // --- 4. Hash and Save ---
        let image = match image::open(&image_path) {
            Ok(img) => img,
            Err(_) => {
                // Silently skip files that can't be opened (e.g., deleted thumbnails)
                continue;
            }
        };

        let hash = hasher.hash_image(&image);
        let Ok(hash): Result<[u8; 8], _> = hash.as_bytes().try_into() else {
            panic!("Hash was not exactly 8 bytes!");
        };
        let hash = i64::from_be_bytes(hash);

        database::save_image(pool, chat_id, &chat_title, msg.id, hash).await?;
    }

    pb.finish_with_message("✅ Import complete!");
    Ok(())
}
