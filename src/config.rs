use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PathConfig {
    pub src: String,
    pub dest: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct WatcherConfig {
    pub name: String,
    pub kube_context: String,
    pub namespace: String,
    pub label_selectors: Vec<String>,
    pub paths: Vec<PathConfig>,
}

pub fn load_configs<P: AsRef<Path>>(path: P) -> Result<Vec<WatcherConfig>> {
    let data = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;
    let configs: Vec<WatcherConfig> =
        serde_json::from_str(&data).context("Failed to parse JSON config")?;
    Ok(configs)
}
