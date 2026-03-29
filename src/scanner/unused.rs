use super::{
    UnusedAsset, UnusedAssetsSummary, UnusedLibrariesSummary, UnusedLibrary, instance_allowed,
};
use crate::prism::CleanupTarget;
use log::warn;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn scan_unused_libraries(root: &Path) -> UnusedLibrariesSummary {
    scan_unused_libraries_scoped(root, None)
}

pub fn scan_unused_libraries_scoped(
    root: &Path,
    selected_instances: Option<&HashSet<String>>,
) -> UnusedLibrariesSummary {
    let libraries_root = root.join("libraries");
    let mut referenced_rel_paths: HashSet<String> = HashSet::new();

    let meta_root = root.join("meta");
    let instances_root = root.join("instances");

    for scan_root in [meta_root, instances_root] {
        if !scan_root.exists() {
            continue;
        }

        for entry in WalkDir::new(scan_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        {
            if let Some(selected) = selected_instances {
                if let Some(instance_name) = extract_instance_name(entry.path())
                    && !instance_allowed(&instance_name, Some(selected))
                {
                    continue;
                }
            }

            let Ok(content) = fs::read_to_string(entry.path()) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
                continue;
            };
            extract_library_paths_from_json(&value, &mut referenced_rel_paths);
        }
    }

    if referenced_rel_paths.is_empty() {
        warn!("no library references were discovered; skipping unused-library candidates");
        return UnusedLibrariesSummary {
            root: root.to_path_buf(),
            candidates: Vec::new(),
            total_bytes: 0,
            referenced_files: 0,
        };
    }

    let mut candidates: Vec<UnusedLibrary> = Vec::new();
    if libraries_root.exists() {
        candidates = WalkDir::new(&libraries_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter_map(|entry| {
                let path = entry.path();
                let rel = path.strip_prefix(&libraries_root).ok()?;
                let rel_norm = rel.to_string_lossy().replace('\\', "/");
                if referenced_rel_paths.contains(&rel_norm) {
                    return None;
                }
                let bytes = entry.metadata().ok()?.len();
                Some(UnusedLibrary {
                    relative_path: rel_norm,
                    path: path.to_path_buf(),
                    bytes,
                })
            })
            .collect();
    }

    candidates.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let total_bytes = candidates.iter().map(|entry| entry.bytes).sum();

    UnusedLibrariesSummary {
        root: root.to_path_buf(),
        candidates,
        total_bytes,
        referenced_files: referenced_rel_paths.len(),
    }
}

pub fn scan_unused_assets(root: &Path) -> UnusedAssetsSummary {
    let indexes_dir = root.join("assets/indexes");
    let objects_dir = root.join("assets/objects");
    let mut used_hashes = HashSet::new();

    if indexes_dir.exists() {
        for entry in WalkDir::new(&indexes_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        {
            let Ok(content) = fs::read_to_string(entry.path()) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
                continue;
            };
            extract_asset_hashes(&value, &mut used_hashes);
        }
    }

    if used_hashes.is_empty() {
        warn!("no asset hashes were discovered; skipping unused-asset candidates");
        return UnusedAssetsSummary {
            root: root.to_path_buf(),
            candidates: Vec::new(),
            total_bytes: 0,
            referenced_hashes: 0,
        };
    }

    let mut candidates: Vec<UnusedAsset> = Vec::new();
    if objects_dir.exists() {
        candidates = WalkDir::new(&objects_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter_map(|entry| {
                let path = entry.path();
                let hash = path.file_name()?.to_string_lossy().to_string();
                if used_hashes.contains(&hash) {
                    return None;
                }
                let bytes = entry.metadata().ok()?.len();
                Some(UnusedAsset {
                    hash,
                    path: path.to_path_buf(),
                    bytes,
                })
            })
            .collect();
    }

    candidates.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    let total_bytes = candidates.iter().map(|entry| entry.bytes).sum();

    UnusedAssetsSummary {
        root: root.to_path_buf(),
        candidates,
        total_bytes,
        referenced_hashes: used_hashes.len(),
    }
}

pub fn cleanup_targets_from_unused_libraries(
    summary: &UnusedLibrariesSummary,
    max_candidates: usize,
) -> Vec<CleanupTarget> {
    summary
        .candidates
        .iter()
        .take(max_candidates)
        .map(|entry| CleanupTarget {
            kind: "advanced".to_string(),
            label: format!("unused-library/{}", entry.relative_path),
            path: entry.path.clone(),
        })
        .collect()
}

pub fn cleanup_targets_from_unused_assets(
    summary: &UnusedAssetsSummary,
    max_candidates: usize,
) -> Vec<CleanupTarget> {
    summary
        .candidates
        .iter()
        .take(max_candidates)
        .map(|entry| CleanupTarget {
            kind: "advanced".to_string(),
            label: format!("unused-asset/{}", entry.hash),
            path: entry.path.clone(),
        })
        .collect()
}

fn extract_library_paths_from_json(value: &serde_json::Value, out: &mut HashSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(artifact_path) = map
                .get("downloads")
                .and_then(|downloads| downloads.get("artifact"))
                .and_then(|artifact| artifact.get("path"))
                .and_then(serde_json::Value::as_str)
            {
                out.insert(artifact_path.replace('\\', "/"));
            }

            if let Some(path) = map.get("path").and_then(serde_json::Value::as_str)
                && path.ends_with(".jar")
                && path.contains('/')
            {
                out.insert(path.replace('\\', "/"));
            }

            for child in map.values() {
                extract_library_paths_from_json(child, out);
            }
        }
        serde_json::Value::Array(arr) => {
            for child in arr {
                extract_library_paths_from_json(child, out);
            }
        }
        _ => {}
    }
}

fn extract_asset_hashes(value: &serde_json::Value, out: &mut HashSet<String>) {
    if let Some(objects) = value.get("objects").and_then(serde_json::Value::as_object) {
        for object in objects.values() {
            if let Some(hash) = object.get("hash").and_then(serde_json::Value::as_str) {
                out.insert(hash.to_string());
            }
        }
    }
}

fn extract_instance_name(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();
    path_str
        .split("/instances/")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .map(|s| s.to_string())
}
