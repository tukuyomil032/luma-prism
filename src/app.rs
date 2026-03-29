use crate::cli::{Cli, Command, Language};
use crate::i18n::{Msg, text};
use crate::{cleaner, cli, config, output, prism, scanner};
use anyhow::{Context, Result};
use clap::Parser;
use dialoguer::{MultiSelect, Select, theme::ColorfulTheme};
use env_logger::Env;
use log::{debug, info};
use serde::Serialize;
use std::collections::HashSet;

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    init_logging(cli.log_level, cli.verbose)?;

    if cli.verbose {
        debug!("arguments: {:?}", cli.command);
    }

    if let Command::Config { lang, show } = cli.command {
        return run_config(lang, show);
    }

    let root = prism::resolve_root(cli.path)?;
    let cfg = config::load_config()?;
    let lang = cfg.language;

    if !root.exists() {
        let message = text(lang, Msg::RootMissing);
        anyhow::bail!("{message}: {}", root.display());
    }

    if cli.verbose {
        let prefix = text(lang, Msg::RootLabel);
        eprintln!("{prefix}: {}", root.display());
    }

    info!("root={}", root.display());

    match cli.command {
        Command::Scan {
            all_instances,
            instances,
            hotspots_depth,
            hotspots_top,
        } => {
            let selected =
                resolve_selected_instances(&root, &instances, all_instances, cli.json, lang)?;

            let mut targets = prism::collect_cleanup_targets(&root);
            filter_cleanup_targets_by_instances(&mut targets, selected.as_ref());

            let summary = run_task(scan_cleanup_msg(lang), !cli.json, lang, || {
                Ok(scanner::scan_cleanup_targets(&root, &targets))
            })?;

            let unused_libraries =
                run_task(scan_unused_libraries_msg(lang), !cli.json, lang, || {
                    Ok(scanner::scan_unused_libraries_scoped(
                        &root,
                        selected.as_ref(),
                    ))
                })?;

            let unused_assets = run_task(scan_unused_assets_msg(lang), !cli.json, lang, || {
                Ok(scanner::scan_unused_assets(&root))
            })?;

            let instance_hotspots =
                run_task(scan_instance_hotspots_msg(lang), !cli.json, lang, || {
                    Ok(scanner::scan_instance_hotspots_scoped(
                        &root,
                        selected.as_ref(),
                        hotspots_depth,
                        hotspots_top,
                    ))
                })?;

            if cli.json {
                let report = ScanJsonReport {
                    cleanup: summary,
                    unused_libraries,
                    unused_assets,
                    instance_hotspots,
                };
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                output::present_scan_report(
                    &summary,
                    &unused_libraries,
                    &unused_assets,
                    &instance_hotspots,
                    lang,
                )?;
            }
        }
        Command::Clean {
            dry_run,
            apply,
            yes,
            include_unused_libraries,
            include_unused_assets,
            kinds,
            min_size,
            older_than_days,
            select,
        } => {
            let min_size_bytes = min_size.as_deref().map(parse_size_to_bytes).transpose()?;

            let mode = cli::CleanMode {
                dry_run: dry_run || !apply,
                yes,
                include_unused_libraries,
                include_unused_assets,
                kinds,
                min_size_bytes,
                older_than_days,
                select,
            };

            let mut targets = prism::collect_cleanup_targets(&root);

            if mode.include_unused_libraries {
                let libs = run_task(scan_unused_libraries_msg(lang), !cli.json, lang, || {
                    Ok(scanner::scan_unused_libraries(&root))
                })?;
                targets.extend(scanner::cleanup_targets_from_unused_libraries(&libs, 2000));
            }

            if mode.include_unused_assets {
                let assets = run_task(scan_unused_assets_msg(lang), !cli.json, lang, || {
                    Ok(scanner::scan_unused_assets(&root))
                })?;
                targets.extend(scanner::cleanup_targets_from_unused_assets(&assets, 5000));
            }

            let filter = cleaner::CleanFilter {
                kinds: mode.kinds.clone(),
                min_size_bytes: mode.min_size_bytes,
                older_than_days: mode.older_than_days,
                interactive_select: mode.select,
            };
            targets = cleaner::filter_and_select_targets(&targets, &filter, lang, !cli.json)?;

            let summary = run_task(clean_targets_msg(lang), !cli.json, lang, || {
                cleaner::run_clean(&root, &targets, mode.dry_run, mode.yes, lang)
            })?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                output::print_clean(&summary, lang);
            }
        }
        Command::Mods => {
            let summary = run_task(scan_duplicate_mods_msg(lang), !cli.json, lang, || {
                Ok(scanner::scan_duplicate_mods(&root))
            })?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                output::print_mods(&summary, lang);
            }
        }
        Command::Worlds { breakdown } => {
            let summary = run_task(scan_worlds_msg(lang), !cli.json, lang, || {
                Ok(scanner::scan_world_sizes_scoped_with_breakdown(
                    &root, None, breakdown,
                ))
            })?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                output::print_worlds(&summary, lang);
            }
        }
        Command::Usage => {
            let summary = run_task(scan_usage_msg(lang), !cli.json, lang, || {
                Ok(scanner::scan_instance_usage(&root))
            })?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                output::print_usage(&summary, lang);
            }
        }
        Command::Config { .. } => unreachable!(),
    }

    Ok(())
}

fn with_status<T, F>(message: &str, lang: Language, f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    print!("{} ... ", message);
    use std::io::Write;
    std::io::stdout()
        .flush()
        .context("failed to flush stdout")?;

    let result = f();
    match &result {
        Ok(_) => println!("{}", text(lang, Msg::StatusDone)),
        Err(_) => println!("{}", text(lang, Msg::StatusFailed)),
    }

    result
}

fn run_task<T, F>(message: &str, show_status: bool, lang: Language, f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    if show_status {
        with_status(message, lang, f)
    } else {
        f()
    }
}

fn init_logging(level: cli::LogLevel, verbose: bool) -> Result<()> {
    let effective = if verbose && level < cli::LogLevel::Debug {
        cli::LogLevel::Debug
    } else {
        level
    };

    let mut builder = env_logger::Builder::from_env(Env::default().default_filter_or("warn"));
    builder.filter_level(effective.as_filter());
    builder.format_timestamp_millis();
    builder.try_init().context("failed to initialize logger")?;
    Ok(())
}

fn run_config(lang: Option<Language>, show: bool) -> Result<()> {
    let mut cfg = config::load_config()?;

    if show {
        println!("{}", serde_json::to_string_pretty(&cfg)?);
        return Ok(());
    }

    if let Some(language) = lang {
        cfg.language = language;
        config::save_config(&cfg)?;
        println!("saved: {}", serde_json::to_string_pretty(&cfg)?);
        return Ok(());
    }

    let theme = ColorfulTheme::default();
    let labels = ["English", "日本語"];
    let default_index = if matches!(cfg.language, Language::Ja) {
        1
    } else {
        0
    };

    let selected = Select::with_theme(&theme)
        .with_prompt(text(cfg.language, Msg::ConfigPromptDefaultLanguage))
        .items(labels)
        .default(default_index)
        .interact()
        .context(text(cfg.language, Msg::ConfigReadSelectionFailed))?;

    cfg.language = if selected == 1 {
        Language::Ja
    } else {
        Language::En
    };
    config::save_config(&cfg)?;

    println!("saved: {}", serde_json::to_string_pretty(&cfg)?);
    Ok(())
}

fn resolve_selected_instances(
    root: &std::path::Path,
    instances: &[String],
    all_instances: bool,
    non_interactive: bool,
    lang: Language,
) -> Result<Option<HashSet<String>>> {
    if all_instances {
        return Ok(None);
    }

    if !instances.is_empty() {
        return Ok(Some(instances.iter().cloned().collect()));
    }

    if non_interactive {
        return Ok(None);
    }

    let available = prism::list_instances(root);
    if available.len() <= 1 {
        return Ok(None);
    }

    let theme = ColorfulTheme::default();
    let prompt = text(lang, Msg::SelectInstancesPrompt);

    let picked = MultiSelect::with_theme(&theme)
        .with_prompt(prompt)
        .items(&available)
        .interact()
        .context(text(lang, Msg::SelectInstancesReadFailed))?;

    if picked.is_empty() {
        return Ok(None);
    }

    let selected = picked
        .into_iter()
        .filter_map(|index| available.get(index).cloned())
        .collect::<HashSet<_>>();

    Ok(Some(selected))
}

fn filter_cleanup_targets_by_instances(
    targets: &mut Vec<prism::CleanupTarget>,
    selected: Option<&HashSet<String>>,
) {
    let Some(selected) = selected else {
        return;
    };

    targets.retain(|target| {
        if target.kind != "instance" {
            return true;
        }

        target
            .label
            .split('/')
            .next()
            .is_some_and(|name| selected.contains(name))
    });
}

fn scan_cleanup_msg(lang: Language) -> &'static str {
    text(lang, Msg::TaskScanCleanup)
}

fn scan_unused_libraries_msg(lang: Language) -> &'static str {
    text(lang, Msg::TaskScanUnusedLibraries)
}

fn scan_unused_assets_msg(lang: Language) -> &'static str {
    text(lang, Msg::TaskScanUnusedAssets)
}

fn scan_instance_hotspots_msg(lang: Language) -> &'static str {
    text(lang, Msg::TaskScanInstanceHotspots)
}

fn clean_targets_msg(lang: Language) -> &'static str {
    text(lang, Msg::TaskCleanTargets)
}

fn scan_duplicate_mods_msg(lang: Language) -> &'static str {
    text(lang, Msg::TaskScanDuplicateMods)
}

fn scan_worlds_msg(lang: Language) -> &'static str {
    text(lang, Msg::TaskScanWorlds)
}

fn scan_usage_msg(lang: Language) -> &'static str {
    text(lang, Msg::TaskScanUsage)
}

fn parse_size_to_bytes(raw: &str) -> Result<u64> {
    let normalized = raw.trim().to_ascii_lowercase();
    let split_at = normalized
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(normalized.len());

    let number = normalized[..split_at].trim();
    let suffix = normalized[split_at..].trim();

    if number.is_empty() {
        anyhow::bail!("invalid size: {raw}");
    }

    let value: f64 = number
        .parse()
        .with_context(|| format!("invalid size number: {raw}"))?;

    let multiplier = match suffix {
        "" | "b" => 1_f64,
        "k" | "kb" | "kib" => 1024_f64,
        "m" | "mb" | "mib" => 1024_f64 * 1024_f64,
        "g" | "gb" | "gib" => 1024_f64 * 1024_f64 * 1024_f64,
        "t" | "tb" | "tib" => 1024_f64 * 1024_f64 * 1024_f64 * 1024_f64,
        _ => anyhow::bail!("unsupported size suffix: {raw}"),
    };

    Ok((value * multiplier) as u64)
}

#[derive(Serialize)]
struct ScanJsonReport {
    cleanup: scanner::CleanupSummary,
    unused_libraries: scanner::UnusedLibrariesSummary,
    unused_assets: scanner::UnusedAssetsSummary,
    instance_hotspots: scanner::InstanceHotspotsSummary,
}
