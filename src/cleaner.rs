use crate::cli::Language;
use crate::i18n::{Msg, text};
use crate::prism::CleanupTarget;
use crate::scanner::dir_size;
use anyhow::{Context, Result};
use dialoguer::{Confirm, MultiSelect, theme::ColorfulTheme};
use indicatif::HumanBytes;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct CleanFilter {
    pub kinds: Vec<String>,
    pub min_size_bytes: Option<u64>,
    pub older_than_days: Option<u64>,
    pub interactive_select: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanEntry {
    pub label: String,
    pub path: String,
    pub bytes: u64,
    pub action: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanSummary {
    pub dry_run: bool,
    pub total_candidates: usize,
    pub total_bytes: u64,
    pub cleaned_bytes: u64,
    pub entries: Vec<CleanEntry>,
}

pub fn filter_and_select_targets(
    targets: &[CleanupTarget],
    filter: &CleanFilter,
    lang: Language,
    allow_interactive: bool,
) -> Result<Vec<CleanupTarget>> {
    let kind_set: HashSet<String> = filter
        .kinds
        .iter()
        .map(|kind| kind.to_lowercase())
        .collect();
    let min_size = filter.min_size_bytes.unwrap_or(0);
    let now = SystemTime::now();
    let older_than = filter
        .older_than_days
        .map(|days| now - Duration::from_secs(days.saturating_mul(86_400)));

    let mut candidates: Vec<(CleanupTarget, u64, Option<SystemTime>)> = targets
        .iter()
        .filter_map(|target| {
            if !target.path.exists() {
                return None;
            }

            if !kind_set.is_empty() && !kind_set.contains(&target.kind.to_lowercase()) {
                return None;
            }

            let bytes = dir_size(&target.path);
            if bytes < min_size {
                return None;
            }

            let modified = target
                .path
                .metadata()
                .ok()
                .and_then(|meta| meta.modified().ok());
            if let Some(threshold) = older_than {
                match modified {
                    Some(mtime) if mtime <= threshold => {}
                    _ => return None,
                }
            }

            Some((target.clone(), bytes, modified))
        })
        .collect();

    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    candidates.sort_by(|a, b| b.1.cmp(&a.1));

    if !filter.interactive_select || !allow_interactive {
        return Ok(candidates.into_iter().map(|row| row.0).collect());
    }

    let theme = ColorfulTheme::default();
    let items: Vec<String> = candidates
        .iter()
        .map(|(target, bytes, modified)| {
            let age = modified
                .and_then(|mtime| now.duration_since(mtime).ok())
                .map(|d| format!("{}d", d.as_secs() / 86_400))
                .unwrap_or_else(|| "-".to_string());
            format!(
                "{} [{}] {} age:{}",
                target.label,
                target.kind,
                HumanBytes(*bytes),
                age
            )
        })
        .collect();

    let defaults = vec![true; items.len()];
    let selected = MultiSelect::with_theme(&theme)
        .with_prompt(text(lang, Msg::CleanSelectPrompt))
        .items(&items)
        .defaults(&defaults)
        .interact()
        .context(text(lang, Msg::CleanSelectReadFailed))?;

    if selected.is_empty() {
        return Ok(Vec::new());
    }

    Ok(selected
        .into_iter()
        .filter_map(|index| candidates.get(index).map(|row| row.0.clone()))
        .collect())
}

pub fn run_clean(
    root: &Path,
    targets: &[CleanupTarget],
    dry_run: bool,
    yes: bool,
    lang: Language,
) -> Result<CleanSummary> {
    let theme = ColorfulTheme::default();

    if !dry_run && !yes {
        let approved = Confirm::with_theme(&theme)
            .with_prompt(text(lang, Msg::CleanConfirmPrompt))
            .default(false)
            .interact()
            .context(text(lang, Msg::CleanConfirmReadFailed))?;

        if !approved {
            return Ok(CleanSummary {
                dry_run,
                total_candidates: 0,
                total_bytes: 0,
                cleaned_bytes: 0,
                entries: Vec::new(),
            });
        }
    }

    let mut entries = Vec::new();
    let mut total_bytes = 0_u64;
    let mut cleaned_bytes = 0_u64;

    for target in targets {
        if !target.path.exists() {
            continue;
        }

        let bytes = dir_size(&target.path);
        total_bytes += bytes;

        let mut entry = CleanEntry {
            label: target.label.clone(),
            path: target.path.display().to_string(),
            bytes,
            action: if dry_run {
                "dry-run".to_string()
            } else {
                "trash".to_string()
            },
            success: true,
            message: String::new(),
        };

        if !is_within_root(root, &target.path) {
            entry.success = false;
            entry.message = text(lang, Msg::CleanPathOutsideRoot).to_string();
            entries.push(entry);
            continue;
        }

        if dry_run {
            entry.message = text(lang, Msg::CleanScheduled).to_string();
            cleaned_bytes += bytes;
            entries.push(entry);
            continue;
        }

        match trash::delete(&target.path) {
            Ok(_) => {
                entry.message = text(lang, Msg::CleanMovedToTrash).to_string();
                cleaned_bytes += bytes;
            }
            Err(err) => {
                entry.success = false;
                entry.message = format!("{}: {err}", text(lang, Msg::CleanFailedPrefix));
            }
        }

        entries.push(entry);
    }

    Ok(CleanSummary {
        dry_run,
        total_candidates: entries.len(),
        total_bytes,
        cleaned_bytes,
        entries,
    })
}

fn is_within_root(root: &Path, path: &Path) -> bool {
    let root = match root.canonicalize() {
        Ok(path) => path,
        Err(_) => return false,
    };

    let path = match path.canonicalize() {
        Ok(path) => path,
        Err(_) => return false,
    };

    path.starts_with(root)
}
