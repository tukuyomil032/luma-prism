use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "luma",
    version,
    about = "Analyze and clean PrismLauncher disk usage"
)]
pub struct Cli {
    /// PrismLauncher root path. Uses OS default when omitted.
    #[arg(long, global = true)]
    pub path: Option<PathBuf>,

    /// Emit JSON output
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub json: bool,

    /// Enable verbose output
    #[arg(long, short = 'v', global = true, action = ArgAction::SetTrue)]
    pub verbose: bool,

    /// Log level
    #[arg(long, global = true, value_enum, default_value_t = LogLevel::Warn)]
    pub log_level: LogLevel,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum, Serialize, Deserialize)]
pub enum Language {
    En,
    Ja,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn as_filter(self) -> log::LevelFilter {
        match self {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Analyze reclaimable storage
    Scan {
        /// Scan all instances without interactive selection
        #[arg(long, action = ArgAction::SetTrue)]
        all_instances: bool,

        /// Restrict scan to specific instances (repeatable)
        #[arg(long = "instance")]
        instances: Vec<String>,

        /// Hotspot aggregation depth (1-6)
        #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(usize))]
        hotspots_depth: usize,

        /// Number of nested hotspot entries to keep per instance (1-200)
        #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(usize))]
        hotspots_top: usize,

        /// Compare hotspot result with previous snapshot and save current snapshot
        #[arg(long, action = ArgAction::SetTrue)]
        hotspots_diff: bool,
    },

    /// Clean targets (dry-run by default)
    Clean {
        /// Explicitly force dry-run mode
        #[arg(long, action = ArgAction::SetTrue)]
        dry_run: bool,

        /// Actually delete files (move to trash)
        #[arg(long, action = ArgAction::SetTrue)]
        apply: bool,

        /// Skip confirmation prompt
        #[arg(long, short = 'y', action = ArgAction::SetTrue)]
        yes: bool,

        /// Include detected unused libraries as clean candidates
        #[arg(long, action = ArgAction::SetTrue)]
        include_unused_libraries: bool,

        /// Include detected unused assets as clean candidates
        #[arg(long, action = ArgAction::SetTrue)]
        include_unused_assets: bool,

        /// Include optional map-cache targets (JourneyMap/Xaero/VoxelMap caches)
        #[arg(long, action = ArgAction::SetTrue)]
        include_map_caches: bool,

        /// Filter by target kind (repeatable: global, instance, advanced)
        #[arg(long = "kind")]
        kinds: Vec<String>,

        /// Minimum size filter (e.g. 500MB, 2GB, 1024)
        #[arg(long)]
        min_size: Option<String>,

        /// Keep only candidates older than N days (by modified time)
        #[arg(long)]
        older_than_days: Option<u64>,

        /// Interactively select filtered candidates before cleaning
        #[arg(long, action = ArgAction::SetTrue)]
        select: bool,
    },

    /// Detect duplicate mods across instances
    Mods,

    /// Analyze world sizes
    Worlds {
        /// Show per-world breakdown (region/playerdata/poi/etc.)
        #[arg(long, action = ArgAction::SetTrue)]
        breakdown: bool,
    },

    /// Show per-instance usage
    Usage,

    /// Manage luma configuration
    Config {
        /// Set default output language
        #[arg(long, value_enum)]
        lang: Option<Language>,

        /// Print current configuration
        #[arg(long, action = ArgAction::SetTrue)]
        show: bool,
    },
}

#[derive(Debug, Clone)]
pub struct CleanMode {
    pub dry_run: bool,
    pub yes: bool,
    pub include_unused_libraries: bool,
    pub include_unused_assets: bool,
    pub include_map_caches: bool,
    pub kinds: Vec<String>,
    pub min_size_bytes: Option<u64>,
    pub older_than_days: Option<u64>,
    pub select: bool,
}
