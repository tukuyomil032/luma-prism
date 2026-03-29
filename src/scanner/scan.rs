use super::{
    CleanupStat, CleanupSummary, DuplicateModEntry, DuplicateModsSummary, HotspotCategory,
    HotspotCategoryStat, HotspotGrowthEntry, HotspotGrowthSummary, InstanceHotspotGroup,
    InstanceHotspotPath, InstanceHotspotsSummary, InstanceUsage, UsageSummary, WorldBreakdownItem,
    WorldStat, WorldsSummary, instance_allowed,
};
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const HOTSPOT_MAX_DEPTH_LIMIT: usize = 6;
const HOTSPOT_TOP_LIMIT: usize = 200;

pub fn scan_cleanup_targets(
    root: &Path,
    targets: &[crate::prism::CleanupTarget],
) -> CleanupSummary {
    let mut entries: Vec<CleanupStat> = targets
        .par_iter()
        .map(|target| CleanupStat {
            kind: target.kind.clone(),
            label: target.label.clone(),
            path: target.path.clone(),
            bytes: dir_size(&target.path),
        })
        .collect();

    entries.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let total_bytes = entries.iter().map(|entry| entry.bytes).sum();

    CleanupSummary {
        root: root.to_path_buf(),
        entries,
        total_bytes,
    }
}

pub fn scan_duplicate_mods(root: &Path) -> DuplicateModsSummary {
    scan_duplicate_mods_scoped(root, None)
}

pub fn scan_duplicate_mods_scoped(
    root: &Path,
    selected_instances: Option<&HashSet<String>>,
) -> DuplicateModsSummary {
    let mut jar_files: Vec<(String, PathBuf)> = Vec::new();
    let instances_dir = root.join("instances");

    if let Ok(entries) = fs::read_dir(&instances_dir) {
        for entry in entries.flatten() {
            let instance_name = entry.file_name().to_string_lossy().to_string();
            if !instance_allowed(&instance_name, selected_instances) {
                continue;
            }

            let mods_dir = entry.path().join(".minecraft/mods");
            if !mods_dir.exists() {
                continue;
            }

            for mod_entry in WalkDir::new(mods_dir)
                .min_depth(1)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let path = mod_entry.path();
                if path.extension().is_some_and(|ext| ext == "jar") {
                    jar_files.push((instance_name.clone(), path.to_path_buf()));
                }
            }
        }
    }

    let hashed: Vec<(String, String, String, u64, PathBuf)> = jar_files
        .par_iter()
        .filter_map(|(instance, path)| {
            let bytes = fs::metadata(path).ok()?.len();
            let data = fs::read(path).ok()?;
            let hash = blake3::hash(&data).to_hex().to_string();
            let mod_name = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown.jar".to_string());
            Some((hash, instance.clone(), mod_name, bytes, path.clone()))
        })
        .collect();

    let mut grouped: HashMap<String, Vec<(String, String, u64, PathBuf)>> = HashMap::new();
    for (hash, instance, mod_name, bytes, path) in hashed {
        grouped
            .entry(hash)
            .or_default()
            .push((instance, mod_name, bytes, path));
    }

    let mut duplicates = Vec::new();
    let mut potential_reclaim_bytes = 0_u64;

    for (hash, values) in grouped {
        if values.len() <= 1 {
            continue;
        }

        let bytes = values[0].2;
        let mod_name = values[0].1.clone();
        let mut instances: Vec<String> = values.iter().map(|v| v.0.clone()).collect();
        instances.sort();
        instances.dedup();

        let mut paths: Vec<PathBuf> = values.iter().map(|v| v.3.clone()).collect();
        paths.sort();

        potential_reclaim_bytes += bytes.saturating_mul((values.len() - 1) as u64);

        duplicates.push(DuplicateModEntry {
            hash,
            mod_name,
            bytes,
            instances,
            paths,
        });
    }

    duplicates.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let duplicate_groups = duplicates.len();

    DuplicateModsSummary {
        root: root.to_path_buf(),
        duplicates,
        duplicate_groups,
        potential_reclaim_bytes,
    }
}

pub fn scan_world_sizes_scoped_with_breakdown(
    root: &Path,
    selected_instances: Option<&HashSet<String>>,
    include_breakdown: bool,
) -> WorldsSummary {
    let mut world_dirs: Vec<(String, String, PathBuf)> = Vec::new();
    let instances_dir = root.join("instances");

    if let Ok(entries) = fs::read_dir(&instances_dir) {
        for entry in entries.flatten() {
            let instance = entry.file_name().to_string_lossy().to_string();
            if !instance_allowed(&instance, selected_instances) {
                continue;
            }

            let saves_dir = entry.path().join(".minecraft/saves");
            if !saves_dir.exists() {
                continue;
            }

            if let Ok(worlds) = fs::read_dir(&saves_dir) {
                for world in worlds.flatten() {
                    let world_path = world.path();
                    let is_world_dir = world_path.is_dir()
                        || fs::metadata(&world_path)
                            .map(|meta| meta.is_dir())
                            .unwrap_or(false);
                    if is_world_dir {
                        let world_name = world.file_name().to_string_lossy().to_string();
                        world_dirs.push((instance.clone(), world_name, world_path));
                    }
                }
            }
        }
    }

    let mut worlds: Vec<WorldStat> = world_dirs
        .par_iter()
        .map(|(instance, world, path)| WorldStat {
            instance: instance.clone(),
            world: world.clone(),
            path: path.clone(),
            bytes: dir_size(path),
            breakdown: if include_breakdown {
                world_breakdown(path)
            } else {
                Vec::new()
            },
        })
        .collect();

    worlds.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let total_world_bytes = worlds.iter().map(|world| world.bytes).sum();

    WorldsSummary {
        root: root.to_path_buf(),
        worlds,
        total_world_bytes,
    }
}

pub fn scan_instance_usage(root: &Path) -> UsageSummary {
    scan_instance_usage_scoped(root, None)
}

pub fn scan_instance_usage_scoped(
    root: &Path,
    selected_instances: Option<&HashSet<String>>,
) -> UsageSummary {
    let mut instances = Vec::new();
    let instances_dir = root.join("instances");

    if let Ok(entries) = fs::read_dir(&instances_dir) {
        instances = entries
            .flatten()
            .filter(|entry| entry.path().is_dir())
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                let path = entry.path();
                (name, path)
            })
            .filter(|(name, _)| instance_allowed(name, selected_instances))
            .collect();
    }

    let mut rows: Vec<InstanceUsage> = instances
        .par_iter()
        .map(|(instance, path)| InstanceUsage {
            instance: instance.clone(),
            path: path.clone(),
            bytes: dir_size(path),
        })
        .collect();

    rows.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let total_bytes = rows.iter().map(|row| row.bytes).sum();

    UsageSummary {
        root: root.to_path_buf(),
        instances: rows,
        total_bytes,
    }
}

pub fn scan_instance_hotspots_scoped(
    root: &Path,
    selected_instances: Option<&HashSet<String>>,
    max_depth: usize,
    top_n_per_instance: usize,
) -> InstanceHotspotsSummary {
    let max_depth = max_depth.clamp(1, HOTSPOT_MAX_DEPTH_LIMIT);
    let top_n_per_instance = top_n_per_instance.clamp(1, HOTSPOT_TOP_LIMIT);
    let mut instances = Vec::new();
    let instances_dir = root.join("instances");

    if let Ok(entries) = fs::read_dir(&instances_dir) {
        instances = entries
            .flatten()
            .filter(|entry| entry.path().is_dir())
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                let mc_dir = entry.path().join(".minecraft");
                (name, mc_dir)
            })
            .filter(|(name, mc_dir)| instance_allowed(name, selected_instances) && mc_dir.exists())
            .collect();
    }

    let mut groups: Vec<InstanceHotspotGroup> = instances
        .par_iter()
        .map(|(instance, mc_dir)| {
            let mut buckets: HashMap<String, u64> = HashMap::new();
            let mut category_buckets: HashMap<HotspotCategory, u64> = HashMap::new();
            let mut total_bytes = 0_u64;

            for entry in WalkDir::new(mc_dir)
                .follow_links(true)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_file())
            {
                let bytes = entry.metadata().ok().map(|meta| meta.len()).unwrap_or(0);
                if bytes == 0 {
                    continue;
                }

                total_bytes += bytes;

                let Ok(rel) = entry.path().strip_prefix(mc_dir) else {
                    continue;
                };

                let parts: Vec<String> = rel
                    .components()
                    .filter_map(path_component_to_string)
                    .collect();

                if parts.is_empty() {
                    continue;
                }

                let category = classify_hotspot_category(&parts);
                *category_buckets.entry(category).or_insert(0) += bytes;

                let levels = usize::min(max_depth, parts.len());
                for depth in 1..=levels {
                    let key = parts[..depth].join("/");
                    *buckets.entry(key).or_insert(0) += bytes;
                }
            }

            let mut entries: Vec<InstanceHotspotPath> = buckets
                .into_iter()
                .map(|(relative_path, bytes)| InstanceHotspotPath {
                    path: mc_dir.join(&relative_path),
                    category: classify_hotspot_category_from_relative_path(&relative_path),
                    relative_path,
                    bytes,
                })
                .collect();

            let categories = sorted_category_stats(category_buckets);

            entries.sort_by(|a, b| {
                b.bytes
                    .cmp(&a.bytes)
                    .then_with(|| a.relative_path.cmp(&b.relative_path))
            });

            let mut depth1_entries: Vec<InstanceHotspotPath> = entries
                .iter()
                .filter(|entry| !entry.relative_path.contains('/'))
                .cloned()
                .collect();
            let mut nested_entries: Vec<InstanceHotspotPath> = entries
                .into_iter()
                .filter(|entry| entry.relative_path.contains('/'))
                .collect();

            if nested_entries.len() > top_n_per_instance {
                nested_entries.truncate(top_n_per_instance);
            }

            depth1_entries.extend(nested_entries);

            InstanceHotspotGroup {
                instance: instance.clone(),
                total_bytes,
                categories,
                entries: depth1_entries,
            }
        })
        .collect();

    groups.sort_by(|a, b| {
        b.total_bytes
            .cmp(&a.total_bytes)
            .then_with(|| a.instance.cmp(&b.instance))
    });

    let total_bytes = groups.iter().map(|group| group.total_bytes).sum();
    let mut summary_categories: HashMap<HotspotCategory, u64> = HashMap::new();

    for group in &groups {
        for stat in &group.categories {
            *summary_categories.entry(stat.category).or_insert(0) += stat.bytes;
        }
    }

    InstanceHotspotsSummary {
        root: root.to_path_buf(),
        max_depth,
        top_n_per_instance,
        categories: sorted_category_stats(summary_categories),
        instances: groups,
        total_bytes,
    }
}

pub fn analyze_hotspot_growth(
    current: &InstanceHotspotsSummary,
    baseline: Option<&InstanceHotspotsSummary>,
    snapshot_path: Option<PathBuf>,
    top_n: usize,
) -> HotspotGrowthSummary {
    let Some(previous) = baseline else {
        return HotspotGrowthSummary {
            snapshot_found: false,
            snapshot_path,
            compared_entries: 0,
            increases: Vec::new(),
            total_growth_bytes: 0,
        };
    };

    let mut previous_map: HashMap<(String, String), u64> = HashMap::new();
    for group in &previous.instances {
        for entry in &group.entries {
            previous_map.insert(
                (group.instance.clone(), entry.relative_path.clone()),
                entry.bytes,
            );
        }
    }

    let mut increases = Vec::new();
    let mut total_growth_bytes = 0_u64;
    let mut compared_entries = 0_usize;

    for group in &current.instances {
        for entry in &group.entries {
            compared_entries += 1;

            let key = (group.instance.clone(), entry.relative_path.clone());
            let previous_bytes = previous_map.get(&key).copied().unwrap_or(0);

            if entry.bytes > previous_bytes {
                let delta_bytes = entry.bytes - previous_bytes;
                total_growth_bytes += delta_bytes;

                increases.push(HotspotGrowthEntry {
                    instance: group.instance.clone(),
                    relative_path: entry.relative_path.clone(),
                    category: entry.category,
                    previous_bytes,
                    current_bytes: entry.bytes,
                    delta_bytes,
                });
            }
        }
    }

    increases.sort_by(|a, b| {
        b.delta_bytes
            .cmp(&a.delta_bytes)
            .then_with(|| b.current_bytes.cmp(&a.current_bytes))
            .then_with(|| a.instance.cmp(&b.instance))
            .then_with(|| a.relative_path.cmp(&b.relative_path))
    });

    if increases.len() > top_n {
        increases.truncate(top_n);
    }

    HotspotGrowthSummary {
        snapshot_found: true,
        snapshot_path,
        compared_entries,
        increases,
        total_growth_bytes,
    }
}

fn path_component_to_string(component: std::path::Component<'_>) -> Option<String> {
    match component {
        std::path::Component::Normal(value) => Some(value.to_string_lossy().to_string()),
        _ => None,
    }
}

fn classify_hotspot_category_from_relative_path(relative_path: &str) -> HotspotCategory {
    let parts: Vec<String> = relative_path
        .split('/')
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect();
    classify_hotspot_category(&parts)
}

fn classify_hotspot_category(parts: &[String]) -> HotspotCategory {
    if parts.is_empty() {
        return HotspotCategory::Unknown;
    }

    let lower_parts: Vec<String> = parts.iter().map(|part| part.to_ascii_lowercase()).collect();
    let first = lower_parts[0].as_str();

    let has_map_keyword = lower_parts.iter().any(|part| {
        part.contains("journeymap")
            || part.contains("xaero")
            || part.contains("voxelmap")
            || part.contains("worldmap")
            || part.contains("minimap")
            || part.contains("ftbchunks")
            || part.contains("dynmap")
            || part.contains("squaremap")
            || part.contains("journey_map")
            || part.contains("xaeroworld")
            || part.contains("atlas")
    });

    let has_media_keyword = lower_parts.iter().any(|part| {
        part.contains("screenshot")
            || part.contains("replay")
            || part.contains("video")
            || part.contains("recording")
            || part.contains("capture")
    });

    let has_cache_keyword = lower_parts.iter().any(|part| {
        part.contains("cache")
            || part == "tmp"
            || part == "temp"
            || part.contains("download")
            || part.contains("checksum")
    });

    let has_log_keyword = lower_parts.iter().any(|part| {
        part == "logs"
            || part.contains("crash-report")
            || part.contains("latest.log")
            || part.ends_with(".log")
    });

    let has_config_keyword = lower_parts.iter().any(|part| {
        part == "options.txt"
            || part == "optionsof.txt"
            || part == "optionsshaders.txt"
            || part == "servers.dat"
            || part.ends_with(".cfg")
            || part.ends_with(".conf")
            || part.ends_with(".ini")
            || part.ends_with(".toml")
            || part.ends_with(".properties")
            || part.ends_with(".json")
    });

    if first == "saves" {
        return if has_map_keyword {
            HotspotCategory::MapData
        } else {
            HotspotCategory::World
        };
    }

    if first == "mods" {
        return HotspotCategory::Mods;
    }

    if first == "config" || (lower_parts.len() == 1 && has_config_keyword) {
        return HotspotCategory::Config;
    }

    if has_config_keyword
        && !has_map_keyword
        && !has_media_keyword
        && !has_cache_keyword
        && !has_log_keyword
    {
        return HotspotCategory::Config;
    }

    if first == "resourcepacks"
        || first == "shaderpacks"
        || first == "assets"
        || first == "resource"
    {
        return HotspotCategory::Resource;
    }

    if first == "logs" || first == "crash-reports" || has_log_keyword {
        return HotspotCategory::Logs;
    }

    if first == "screenshots"
        || first == "replay_recordings"
        || first == "replay_videos"
        || (has_media_keyword && first != "essential")
    {
        return HotspotCategory::Media;
    }

    if first == "journeymap"
        || first.contains("xaero")
        || first.contains("voxelmap")
        || first == "litematica"
        || first == "schematics"
        || (first == "local" && has_map_keyword)
        || has_map_keyword
    {
        return HotspotCategory::MapData;
    }

    if first == "essential" {
        return if has_cache_keyword {
            HotspotCategory::ModCache
        } else if has_media_keyword {
            HotspotCategory::Media
        } else {
            HotspotCategory::ModCache
        };
    }

    if first == ".replay_cache" || has_cache_keyword {
        return HotspotCategory::ModCache;
    }

    HotspotCategory::Unknown
}

fn sorted_category_stats(
    category_buckets: HashMap<HotspotCategory, u64>,
) -> Vec<HotspotCategoryStat> {
    let mut categories: Vec<HotspotCategoryStat> = category_buckets
        .into_iter()
        .filter(|(_, bytes)| *bytes > 0)
        .map(|(category, bytes)| HotspotCategoryStat { category, bytes })
        .collect();

    categories.sort_by(|a, b| {
        b.bytes
            .cmp(&a.bytes)
            .then_with(|| a.category.as_label().cmp(b.category.as_label()))
    });

    categories
}

fn world_breakdown(world_path: &Path) -> Vec<WorldBreakdownItem> {
    let mut buckets: BTreeMap<String, u64> = BTreeMap::new();

    let Ok(entries) = fs::read_dir(world_path) else {
        return Vec::new();
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let bytes = if path.is_dir() {
            dir_size(&path)
        } else {
            fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
        };
        if bytes == 0 {
            continue;
        }

        let bucket = match name.as_str() {
            "region" | "playerdata" | "poi" | "data" | "entities" | "advancements" | "stats" => {
                name.clone()
            }
            "DIM-1" | "DIM1" | "dimensions" => name.clone(),
            _ if name.starts_with("DIM") => name.clone(),
            _ => "other".to_string(),
        };

        *buckets.entry(bucket).or_insert(0) += bytes;
    }

    let mut items: Vec<WorldBreakdownItem> = buckets
        .into_iter()
        .map(|(bucket, bytes)| WorldBreakdownItem { bucket, bytes })
        .collect();
    items.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    items
}

pub fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| entry.metadata().ok().map(|meta| meta.len()))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{
        HotspotCategory, HotspotCategoryStat, InstanceHotspotGroup, InstanceHotspotPath,
        InstanceHotspotsSummary, analyze_hotspot_growth,
        classify_hotspot_category_from_relative_path,
    };
    use std::path::PathBuf;

    #[test]
    fn classify_world_path() {
        let category =
            classify_hotspot_category_from_relative_path("saves/survival/region/r.0.0.mca");
        assert_eq!(category, HotspotCategory::World);
    }

    #[test]
    fn classify_map_data_path() {
        let category =
            classify_hotspot_category_from_relative_path("saves/survival/ftbchunks/data/0,0.dat");
        assert_eq!(category, HotspotCategory::MapData);
    }

    #[test]
    fn classify_mod_cache_path() {
        let category =
            classify_hotspot_category_from_relative_path("essential/screenshot-cache/chunk-1.bin");
        assert_eq!(category, HotspotCategory::ModCache);
    }

    #[test]
    fn classify_media_path() {
        let category =
            classify_hotspot_category_from_relative_path("screenshots/2026-03-29_12.00.00.png");
        assert_eq!(category, HotspotCategory::Media);
    }

    #[test]
    fn classify_logs_path() {
        let category = classify_hotspot_category_from_relative_path("logs/latest.log");
        assert_eq!(category, HotspotCategory::Logs);
    }

    #[test]
    fn classify_resource_path() {
        let category =
            classify_hotspot_category_from_relative_path("shaderpacks/Complementary.zip");
        assert_eq!(category, HotspotCategory::Resource);
    }

    #[test]
    fn classify_mods_path() {
        let category = classify_hotspot_category_from_relative_path("mods/sodium-fabric.jar");
        assert_eq!(category, HotspotCategory::Mods);
    }

    #[test]
    fn classify_config_path() {
        let category = classify_hotspot_category_from_relative_path("config/sodium-options.json");
        assert_eq!(category, HotspotCategory::Config);
    }

    #[test]
    fn classify_root_options_file_as_config() {
        let category = classify_hotspot_category_from_relative_path("options.txt");
        assert_eq!(category, HotspotCategory::Config);
    }

    #[test]
    fn classify_root_servers_file_as_config() {
        let category = classify_hotspot_category_from_relative_path("servers.dat");
        assert_eq!(category, HotspotCategory::Config);
    }

    #[test]
    fn classify_unknown_path() {
        let category = classify_hotspot_category_from_relative_path("foo/bar/data.bin");
        assert_eq!(category, HotspotCategory::Unknown);
    }

    #[test]
    fn hotspot_growth_detects_increases_only() {
        let baseline = sample_hotspots("pack", vec![("saves/world", HotspotCategory::World, 100)]);
        let current = sample_hotspots(
            "pack",
            vec![
                ("saves/world", HotspotCategory::World, 140),
                ("screenshots", HotspotCategory::Media, 60),
            ],
        );

        let growth = analyze_hotspot_growth(&current, Some(&baseline), None, 10);
        assert!(growth.snapshot_found);
        assert_eq!(growth.compared_entries, 2);
        assert_eq!(growth.increases.len(), 2);
        assert_eq!(growth.total_growth_bytes, 100);
        assert_eq!(growth.increases[0].delta_bytes, 60);
        assert_eq!(growth.increases[1].delta_bytes, 40);
    }

    #[test]
    fn hotspot_growth_without_baseline() {
        let current = sample_hotspots("pack", vec![("logs", HotspotCategory::Logs, 32)]);
        let growth = analyze_hotspot_growth(&current, None, None, 10);
        assert!(!growth.snapshot_found);
        assert_eq!(growth.compared_entries, 0);
        assert!(growth.increases.is_empty());
        assert_eq!(growth.total_growth_bytes, 0);
    }

    fn sample_hotspots(
        instance: &str,
        entries: Vec<(&str, HotspotCategory, u64)>,
    ) -> InstanceHotspotsSummary {
        let total_bytes = entries.iter().map(|(_, _, bytes)| *bytes).sum::<u64>();
        let hotspot_entries = entries
            .iter()
            .map(|(relative_path, category, bytes)| InstanceHotspotPath {
                relative_path: (*relative_path).to_string(),
                path: PathBuf::from(format!("/tmp/{instance}/{relative_path}")),
                category: *category,
                bytes: *bytes,
            })
            .collect::<Vec<_>>();

        let categories = vec![HotspotCategoryStat {
            category: HotspotCategory::Unknown,
            bytes: total_bytes,
        }];

        InstanceHotspotsSummary {
            root: PathBuf::from("/tmp"),
            max_depth: 2,
            top_n_per_instance: 30,
            categories: categories.clone(),
            instances: vec![InstanceHotspotGroup {
                instance: instance.to_string(),
                total_bytes,
                categories,
                entries: hotspot_entries,
            }],
            total_bytes,
        }
    }
}
