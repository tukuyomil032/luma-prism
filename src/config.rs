use crate::cli::Language;
use crate::scanner::InstanceHotspotsSummary;
use anyhow::{Context, Result};
use log::warn;
use serde::{Deserialize, Serialize};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: Language,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: Language::En,
        }
    }
}

pub fn load_config() -> Result<AppConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config: {}", path.display()))?;
    let cfg = serde_json::from_str::<AppConfig>(&content)
        .with_context(|| format!("failed to parse config: {}", path.display()))?;
    Ok(cfg)
}

pub fn save_config(cfg: &AppConfig) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config dir: {}", parent.display()))?;
    }

    let body = serde_json::to_string_pretty(cfg)?;
    fs::write(&path, body).with_context(|| format!("failed to write config: {}", path.display()))
}

pub fn config_path() -> Result<PathBuf> {
    let base = dirs::config_dir().context("failed to resolve config dir")?;
    Ok(base.join("luma-prism").join("config.json"))
}

pub fn hotspot_snapshot_path(root: &Path) -> Result<PathBuf> {
    let base = dirs::config_dir().context("failed to resolve config dir")?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    root.to_string_lossy().hash(&mut hasher);
    let id = hasher.finish();
    Ok(base
        .join("luma-prism")
        .join("snapshots")
        .join(format!("hotspots-{id:016x}.json")))
}

pub fn load_hotspot_snapshot(root: &Path) -> Result<Option<InstanceHotspotsSummary>> {
    let path = hotspot_snapshot_path(root)?;
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read hotspot snapshot: {}", path.display()))?;

    match serde_json::from_str::<InstanceHotspotsSummary>(&content) {
        Ok(snapshot) => Ok(Some(snapshot)),
        Err(err) => {
            warn!(
                "failed to parse hotspot snapshot (ignored): path={}, err={}",
                path.display(),
                err
            );
            Ok(None)
        }
    }
}

pub fn save_hotspot_snapshot(root: &Path, summary: &InstanceHotspotsSummary) -> Result<PathBuf> {
    let path = hotspot_snapshot_path(root)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create snapshot dir: {}", parent.display()))?;
    }

    let body = serde_json::to_string_pretty(summary)?;
    fs::write(&path, body)
        .with_context(|| format!("failed to write hotspot snapshot: {}", path.display()))?;

    Ok(path)
}
