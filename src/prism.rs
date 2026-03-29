use anyhow::{Context, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct CleanupTarget {
    pub kind: String,
    pub label: String,
    pub path: PathBuf,
}

pub fn resolve_root(explicit: Option<PathBuf>) -> Result<PathBuf> {
    let root = match explicit {
        Some(path) => path,
        None => default_prism_root().context("failed to resolve default PrismLauncher root")?,
    };

    let normalized = root
        .canonicalize()
        .with_context(|| format!("failed to resolve path: {}", root.display()))?;

    Ok(normalized)
}

pub fn default_prism_root() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir()?;
        return Some(home.join("Library/Application Support/PrismLauncher"));
    }

    #[cfg(target_os = "windows")]
    {
        let roaming = dirs::config_dir()?;
        return Some(roaming.join("PrismLauncher"));
    }

    #[allow(unreachable_code)]
    None
}

pub fn collect_cleanup_targets(root: &Path) -> Vec<CleanupTarget> {
    let mut targets = Vec::new();

    let push_if_exists =
        |targets: &mut Vec<CleanupTarget>, kind: &str, label: &str, path: PathBuf| {
            if path.exists() {
                targets.push(CleanupTarget {
                    kind: kind.to_string(),
                    label: label.to_string(),
                    path,
                });
            }
        };

    push_if_exists(&mut targets, "global", "cache", root.join("cache"));
    push_if_exists(&mut targets, "global", "logs", root.join("logs"));
    push_if_exists(&mut targets, "global", "meta", root.join("meta"));
    push_if_exists(&mut targets, "global", "catpacks", root.join("catpacks"));

    let instances_dir = root.join("instances");
    if let Ok(entries) = std::fs::read_dir(instances_dir) {
        for entry in entries.flatten() {
            let instance_path = entry.path();
            if !instance_path.is_dir() {
                continue;
            }

            let instance_name = entry.file_name().to_string_lossy().to_string();
            let mc = instance_path.join(".minecraft");
            push_if_exists(
                &mut targets,
                "instance",
                &format!("{instance_name}/logs"),
                mc.join("logs"),
            );
            push_if_exists(
                &mut targets,
                "instance",
                &format!("{instance_name}/crash-reports"),
                mc.join("crash-reports"),
            );

            // Known regenerable mod caches discovered from Top500 usage patterns.
            push_if_exists(
                &mut targets,
                "instance",
                &format!("{instance_name}/.replay_cache"),
                mc.join(".replay_cache"),
            );
            push_if_exists(
                &mut targets,
                "instance",
                &format!("{instance_name}/essential/screenshot-cache"),
                mc.join("essential/screenshot-cache"),
            );
            push_if_exists(
                &mut targets,
                "instance",
                &format!("{instance_name}/essential/cosmetic-cache"),
                mc.join("essential/cosmetic-cache"),
            );
            push_if_exists(
                &mut targets,
                "instance",
                &format!("{instance_name}/essential/screenshot-checksum-caches.json"),
                mc.join("essential/screenshot-checksum-caches.json"),
            );
        }
    }

    targets
}

pub fn collect_map_cache_targets(root: &Path) -> Vec<CleanupTarget> {
    let mut targets = Vec::new();

    let push_if_exists =
        |targets: &mut Vec<CleanupTarget>, kind: &str, label: &str, path: PathBuf| {
            if path.exists() {
                targets.push(CleanupTarget {
                    kind: kind.to_string(),
                    label: label.to_string(),
                    path,
                });
            }
        };

    let instances_dir = root.join("instances");
    if let Ok(entries) = std::fs::read_dir(instances_dir) {
        for entry in entries.flatten() {
            let instance_path = entry.path();
            if !instance_path.is_dir() {
                continue;
            }

            let instance_name = entry.file_name().to_string_lossy().to_string();
            let mc = instance_path.join(".minecraft");

            // Optional candidates: potentially large map tiles that can be rebuilt.
            push_if_exists(
                &mut targets,
                "advanced",
                &format!("{instance_name}/journeymap/cache"),
                mc.join("journeymap/cache"),
            );
            push_if_exists(
                &mut targets,
                "advanced",
                &format!("{instance_name}/journeymap/webmap"),
                mc.join("journeymap/webmap"),
            );
            push_if_exists(
                &mut targets,
                "advanced",
                &format!("{instance_name}/xaerominimap/cache"),
                mc.join("xaerominimap/cache"),
            );
            push_if_exists(
                &mut targets,
                "advanced",
                &format!("{instance_name}/xaeroworldmap/cache"),
                mc.join("xaeroworldmap/cache"),
            );
            push_if_exists(
                &mut targets,
                "advanced",
                &format!("{instance_name}/voxelmap/cache"),
                mc.join("voxelmap/cache"),
            );
        }
    }

    targets
}

pub fn list_instances(root: &Path) -> Vec<String> {
    let instances_dir = root.join("instances");
    let Ok(entries) = std::fs::read_dir(instances_dir) else {
        return Vec::new();
    };

    let mut names: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::{collect_cleanup_targets, collect_map_cache_targets};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn collect_cleanup_targets_includes_known_mod_caches() {
        let root = create_temp_root("luma-prism-targets");
        let mc_root = root.join("instances/pack/.minecraft");

        fs::create_dir_all(mc_root.join("logs")).expect("create logs");
        fs::create_dir_all(mc_root.join("crash-reports")).expect("create crash-reports");
        fs::create_dir_all(mc_root.join(".replay_cache")).expect("create .replay_cache");
        fs::create_dir_all(mc_root.join("essential/screenshot-cache"))
            .expect("create screenshot-cache");
        fs::create_dir_all(mc_root.join("essential/cosmetic-cache"))
            .expect("create cosmetic-cache");
        fs::write(
            mc_root.join("essential/screenshot-checksum-caches.json"),
            b"{}",
        )
        .expect("create checksum cache index");

        let targets = collect_cleanup_targets(&root);
        let labels: Vec<String> = targets.iter().map(|target| target.label.clone()).collect();

        assert!(labels.contains(&"pack/.replay_cache".to_string()));
        assert!(labels.contains(&"pack/essential/screenshot-cache".to_string()));
        assert!(labels.contains(&"pack/essential/cosmetic-cache".to_string()));
        assert!(labels.contains(&"pack/essential/screenshot-checksum-caches.json".to_string()));

        fs::remove_dir_all(&root).expect("cleanup temp root");
    }

    #[test]
    fn collect_map_cache_targets_requires_opt_in_paths() {
        let root = create_temp_root("luma-prism-map-cache-targets");
        let mc_root = root.join("instances/pack/.minecraft");

        fs::create_dir_all(mc_root.join("journeymap/cache")).expect("create journeymap cache");
        fs::create_dir_all(mc_root.join("xaerominimap/cache")).expect("create xaero minimap");

        let targets = collect_map_cache_targets(&root);
        let labels: Vec<String> = targets.iter().map(|target| target.label.clone()).collect();

        assert!(labels.contains(&"pack/journeymap/cache".to_string()));
        assert!(labels.contains(&"pack/xaerominimap/cache".to_string()));
        assert!(
            targets
                .iter()
                .all(|target| target.kind == "advanced" && target.label.starts_with("pack/"))
        );

        fs::remove_dir_all(&root).expect("cleanup temp root");
    }

    fn create_temp_root(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
        fs::create_dir_all(&root).expect("create temp root");
        root
    }
}
