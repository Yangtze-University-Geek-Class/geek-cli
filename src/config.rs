use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Stored {
    pub token: Option<String>,
    pub login: Option<String>,
}

fn config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("could not resolve user config directory")?
        .join("geek");
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    Ok(dir)
}

pub fn token_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("token.json"))
}

pub fn load() -> Result<Stored> {
    let path = token_path()?;
    if !path.exists() {
        return Ok(Stored::default());
    }
    let bytes =
        std::fs::read(&path).with_context(|| format!("read token from {}", path.display()))?;
    let parsed: Stored = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(parsed)
}

pub fn require_token() -> Result<String> {
    if let Ok(t) = std::env::var("GEEK_TOKEN") {
        if !t.is_empty() {
            return Ok(t);
        }
    }
    let stored = load()?;
    stored
        .token
        .filter(|t| !t.is_empty())
        .context("not signed in — run `geek login` first")
}
