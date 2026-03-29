use crate::cli::Language;
use crate::i18n::{Msg, text};
use crate::scanner::{
    CleanupSummary, HotspotGrowthSummary, InstanceHotspotsSummary, UnusedAssetsSummary,
    UnusedLibrariesSummary,
};
use anyhow::Result;
use console::{Key, Term, style, truncate_str};
use indicatif::HumanBytes;

pub fn human_bytes(bytes: u64) -> String {
    HumanBytes(bytes).to_string()
}

pub fn present_scan_report(
    summary: &CleanupSummary,
    unused_libs: &UnusedLibrariesSummary,
    unused_assets: &UnusedAssetsSummary,
    hotspots: &InstanceHotspotsSummary,
    hotspot_growth: Option<&HotspotGrowthSummary>,
    lang: Language,
) -> Result<()> {
    let lines = build_scan_lines(
        summary,
        unused_libs,
        unused_assets,
        hotspots,
        hotspot_growth,
        lang,
    );
    pager(&lines, lang)
}

fn build_scan_lines(
    summary: &CleanupSummary,
    unused_libs: &UnusedLibrariesSummary,
    unused_assets: &UnusedAssetsSummary,
    hotspots: &InstanceHotspotsSummary,
    hotspot_growth: Option<&HotspotGrowthSummary>,
    lang: Language,
) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push(match lang {
        Language::En => format!("{}", style(text(lang, Msg::ScanTitle)).bold().cyan()),
        Language::Ja => format!("{}", style(text(lang, Msg::ScanTitle)).bold().cyan()),
    });
    lines.push(String::new());

    lines.push(text(lang, Msg::ScanSafeTargets).to_string());
    for entry in &summary.entries {
        let rel = to_relative(&summary.root, &entry.path);
        lines.push(format!(
            "{:<18} {:>10}  {}",
            entry.label,
            human_bytes(entry.bytes),
            rel
        ));
    }
    lines.push(format!(
        "{}: {}",
        text(lang, Msg::ScanSafeTotal),
        human_bytes(summary.total_bytes)
    ));
    lines.push(String::new());

    lines.push(text(lang, Msg::ScanUnusedLibraries).to_string());
    if unused_libs.candidates.is_empty() {
        lines.push(text(lang, Msg::ScanNone).to_string());
    } else {
        for entry in &unused_libs.candidates {
            lines.push(format!(
                "{:>10}  {}",
                human_bytes(entry.bytes),
                entry.relative_path
            ));
        }
    }
    lines.push(format!(
        "{}: {}",
        text(lang, Msg::ScanUnusedLibrariesTotal),
        human_bytes(unused_libs.total_bytes)
    ));
    lines.push(String::new());

    lines.push(text(lang, Msg::ScanUnusedAssets).to_string());
    if unused_assets.candidates.is_empty() {
        lines.push(text(lang, Msg::ScanNone).to_string());
    } else {
        for entry in &unused_assets.candidates {
            let rel = to_relative(&unused_assets.root, &entry.path);
            lines.push(format!("{:>10}  {}", human_bytes(entry.bytes), rel));
        }
    }
    lines.push(format!(
        "{}: {}",
        text(lang, Msg::ScanUnusedAssetsTotal),
        human_bytes(unused_assets.total_bytes)
    ));

    lines.push(String::new());
    lines.push(text(lang, Msg::ScanInstanceHotspots).to_string());

    lines.push(text(lang, Msg::ScanInstanceHotspotsByCategory).to_string());
    if hotspots.categories.is_empty() {
        lines.push(text(lang, Msg::ScanNone).to_string());
    } else {
        for stat in &hotspots.categories {
            let ratio = if hotspots.total_bytes > 0 {
                (stat.bytes as f64 / hotspots.total_bytes as f64) * 100.0
            } else {
                0.0
            };
            lines.push(format!(
                "  {:>6.1}% {:>10}  [{}]",
                ratio,
                human_bytes(stat.bytes),
                stat.category.as_label()
            ));
        }
    }

    lines.push(String::new());

    if hotspots.instances.is_empty() {
        lines.push(text(lang, Msg::ScanNone).to_string());
    } else {
        for group in &hotspots.instances {
            lines.push(format!(
                "{} ({})",
                group.instance,
                human_bytes(group.total_bytes)
            ));

            if group.entries.is_empty() {
                lines.push(format!("  {}", text(lang, Msg::ScanNone)));
                lines.push(String::new());
                continue;
            }

            if !group.categories.is_empty() {
                lines.push(format!(
                    "  {}",
                    text(lang, Msg::ScanInstanceHotspotsInstanceTags)
                ));
                for stat in &group.categories {
                    let ratio = if group.total_bytes > 0 {
                        (stat.bytes as f64 / group.total_bytes as f64) * 100.0
                    } else {
                        0.0
                    };
                    lines.push(format!(
                        "    {:>6.1}% {:>10}  [{}]",
                        ratio,
                        human_bytes(stat.bytes),
                        stat.category.as_label()
                    ));
                }
            }

            for entry in &group.entries {
                let ratio = if group.total_bytes > 0 {
                    (entry.bytes as f64 / group.total_bytes as f64) * 100.0
                } else {
                    0.0
                };

                lines.push(format!(
                    "  {:>6.1}% {:>10}  [{}] {}",
                    ratio,
                    human_bytes(entry.bytes),
                    entry.category.as_label(),
                    entry.relative_path
                ));
            }

            lines.push(String::new());
        }
    }

    lines.push(format!(
        "{}: {}",
        text(lang, Msg::ScanInstanceHotspotsTotal),
        human_bytes(hotspots.total_bytes)
    ));

    if let Some(growth) = hotspot_growth {
        lines.push(String::new());
        lines.push(text(lang, Msg::ScanHotspotGrowth).to_string());

        if !growth.snapshot_found {
            lines.push(text(lang, Msg::ScanHotspotGrowthNoBaseline).to_string());
        } else {
            lines.push(format!(
                "{}: {}",
                text(lang, Msg::ScanHotspotGrowthComparedEntries),
                growth.compared_entries
            ));

            if growth.increases.is_empty() {
                lines.push(text(lang, Msg::ScanNone).to_string());
            } else {
                for entry in &growth.increases {
                    lines.push(format!(
                        "  +{:>10} [{:>10} -> {:>10}] [{}] {}/{}",
                        human_bytes(entry.delta_bytes),
                        human_bytes(entry.previous_bytes),
                        human_bytes(entry.current_bytes),
                        entry.category.as_label(),
                        entry.instance,
                        entry.relative_path
                    ));
                }
            }
        }

        lines.push(format!(
            "{}: {}",
            text(lang, Msg::ScanHotspotGrowthTotal),
            human_bytes(growth.total_growth_bytes)
        ));
    }

    lines
}

fn to_relative(root: &std::path::Path, path: &std::path::Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

fn pager(lines: &[String], lang: Language) -> Result<()> {
    let term = Term::stdout();
    if !term.is_term() {
        for line in lines {
            println!("{}", line);
        }
        return Ok(());
    }

    let mut page = 0usize;

    loop {
        let (rows, cols) = term.size();
        let page_height = (rows as usize).saturating_sub(3).max(1);
        let total_pages = lines.len().div_ceil(page_height).max(1);
        if page >= total_pages {
            page = total_pages - 1;
        }

        let start = page * page_height;
        let end = usize::min(start + page_height, lines.len());

        term.clear_screen()?;
        for line in &lines[start..end] {
            let clipped = truncate_str(line, cols as usize, "...");
            term.write_line(&clipped)?;
        }

        let help = text(lang, Msg::PagerHelp)
            .replace("{page}", &(page + 1).to_string())
            .replace("{total}", &total_pages.to_string());
        term.write_line(&style(help).dim().to_string())?;

        match term.read_key()? {
            Key::ArrowRight | Key::ArrowDown | Key::Char('j') | Key::Char('l') => {
                if page + 1 < total_pages {
                    page += 1;
                }
            }
            Key::ArrowLeft | Key::ArrowUp | Key::Char('h') | Key::Char('k') => {
                page = page.saturating_sub(1);
            }
            Key::Enter | Key::Escape | Key::Char('q') => {
                term.clear_screen()?;
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
