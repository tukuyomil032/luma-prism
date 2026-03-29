mod scan;
mod unused;

use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct CleanupStat {
    pub kind: String,
    pub label: String,
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanupSummary {
    pub root: PathBuf,
    pub entries: Vec<CleanupStat>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateModEntry {
    pub hash: String,
    pub mod_name: String,
    pub bytes: u64,
    pub instances: Vec<String>,
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateModsSummary {
    pub root: PathBuf,
    pub duplicates: Vec<DuplicateModEntry>,
    pub duplicate_groups: usize,
    pub potential_reclaim_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldStat {
    pub instance: String,
    pub world: String,
    pub path: PathBuf,
    pub bytes: u64,
    pub breakdown: Vec<WorldBreakdownItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldBreakdownItem {
    pub bucket: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldsSummary {
    pub root: PathBuf,
    pub worlds: Vec<WorldStat>,
    pub total_world_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstanceUsage {
    pub instance: String,
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageSummary {
    pub root: PathBuf,
    pub instances: Vec<InstanceUsage>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstanceHotspotPath {
    pub relative_path: String,
    pub path: PathBuf,
    pub category: HotspotCategory,
    pub bytes: u64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HotspotCategory {
    World,
    Media,
    MapData,
    ModCache,
    Logs,
    Resource,
    Mods,
    Config,
    Unknown,
}

impl HotspotCategory {
    pub fn as_label(self) -> &'static str {
        match self {
            HotspotCategory::World => "world",
            HotspotCategory::Media => "media",
            HotspotCategory::MapData => "map-data",
            HotspotCategory::ModCache => "mod-cache",
            HotspotCategory::Logs => "logs",
            HotspotCategory::Resource => "resource",
            HotspotCategory::Mods => "mods",
            HotspotCategory::Config => "config",
            HotspotCategory::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HotspotCategoryStat {
    pub category: HotspotCategory,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstanceHotspotGroup {
    pub instance: String,
    pub total_bytes: u64,
    pub categories: Vec<HotspotCategoryStat>,
    pub entries: Vec<InstanceHotspotPath>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstanceHotspotsSummary {
    pub root: PathBuf,
    pub max_depth: usize,
    pub top_n_per_instance: usize,
    pub categories: Vec<HotspotCategoryStat>,
    pub instances: Vec<InstanceHotspotGroup>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnusedLibrary {
    pub relative_path: String,
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnusedLibrariesSummary {
    pub root: PathBuf,
    pub candidates: Vec<UnusedLibrary>,
    pub total_bytes: u64,
    pub referenced_files: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnusedAsset {
    pub hash: String,
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnusedAssetsSummary {
    pub root: PathBuf,
    pub candidates: Vec<UnusedAsset>,
    pub total_bytes: u64,
    pub referenced_hashes: usize,
}

pub use scan::{
    dir_size, scan_cleanup_targets, scan_duplicate_mods, scan_instance_hotspots_scoped,
    scan_instance_usage, scan_world_sizes_scoped_with_breakdown,
};
pub use unused::{
    cleanup_targets_from_unused_assets, cleanup_targets_from_unused_libraries, scan_unused_assets,
    scan_unused_libraries, scan_unused_libraries_scoped,
};

fn instance_allowed(name: &str, selected: Option<&HashSet<String>>) -> bool {
    selected.is_none_or(|set| set.contains(name))
}
