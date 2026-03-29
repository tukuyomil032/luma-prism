<div align="center">
  <h1>luma-prism</h1>
  <p><strong>Fast PrismLauncher storage analysis and safe cleanup CLI</strong></p>

  <p>
    <a href="https://github.com/tukuyomil032/luma-prism/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/tukuyomil032/luma-prism/ci.yml?branch=master&label=CI"></a>
    <a href="https://github.com/tukuyomil032/luma-prism/actions/workflows/release.yml"><img alt="Release" src="https://img.shields.io/github/actions/workflow/status/tukuyomil032/luma-prism/release.yml?branch=master&label=release"></a>
    <a href="https://github.com/tukuyomil032/luma-prism/releases"><img alt="Downloads" src="https://img.shields.io/github/downloads/tukuyomil032/luma-prism/total"></a>
    <a href="./LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-green"></a>
    <a href="https://www.rust-lang.org/"><img alt="Rust" src="https://img.shields.io/badge/rust-2024-orange"></a>
  </p>
</div>

luma-prism scans PrismLauncher data and helps you reclaim space safely.

It focuses on:
- safe cleanup targets (`cache`, `logs`, `meta`, instance logs/crash reports, known regenerable mod caches)
- full instance hotspot analysis (all `.minecraft` data, not only cleanup-safe paths)
- duplicate mod detection
- world size analysis
- per-instance usage summaries
- optional candidates for unused libraries/assets

By default, cleanup runs in dry-run mode and deletion uses the system trash.

## Features

- Fast parallel scanning (`rayon` + `walkdir`)
- English/Japanese output switch via `luma config`
- Interactive instance selection for `scan`
- Paged scan report viewer
- World breakdown mode (`region`, `playerdata`, `poi`, etc.)
- Instance hotspot breakdown in `scan` (depth-based path aggregation)
- Hotspot category tagging (`world`, `media`, `map-data`, `mod-cache`, `logs`, `resource`, etc.)
- Hotspot snapshot diff (`--hotspots-diff`) to highlight growth since previous scan
- Clean preview filtering by kind/size/age and optional interactive candidate selection

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
- [Release Automation](#release-automation)
- [Safety](#safety)

## Installation

### 1) One-command installer (macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/tukuyomil032/luma-prism/master/install.sh | sh
```

Options:

- Pin a version: `LUMA_VERSION=0.1.0 ...`
- Change install directory: `LUMA_BIN_DIR=$HOME/.local/luma-prism/bin ...`

Local test before push:

```bash
cargo build --release
tar -C target/release -czf /tmp/luma-prism-local-macos.tar.gz luma
cat install.sh | LUMA_BIN_DIR=/tmp/luma-prism-test/bin LUMA_ASSET_URL=file:///tmp/luma-prism-local-macos.tar.gz sh
```

### 2) One-command installer (Windows PowerShell)

```powershell
iwr -useb https://raw.githubusercontent.com/tukuyomil032/luma-prism/master/install.ps1 | iex
```

Options:

- Pin a version: `$env:LUMA_VERSION='0.1.0'`
- Change install directory: `$env:LUMA_BIN_DIR='C:\\tools\\luma-prism\\bin'`

Local test before push:

```powershell
cargo build --release
Compress-Archive -Path .\target\release\luma.exe -DestinationPath $env:TEMP\luma-prism-local-win.zip -Force
$env:LUMA_ASSET_URL = "file:///$($env:TEMP -replace '\\','/')/luma-prism-local-win.zip"
Get-Content .\install.ps1 -Raw | Invoke-Expression
```

### 3) GitHub Releases binaries

Download prebuilt binaries from:

- https://github.com/tukuyomil032/luma-prism/releases

### 4) cargo install

From git (works now):

```bash
cargo install --git https://github.com/tukuyomil032/luma-prism luma-prism
```

From crates.io (after publish):

```bash
cargo install luma-prism
```

### 5) Build from source

```bash
git clone https://github.com/tukuyomil032/luma-prism
cd luma-prism
cargo build --release
```

## Uninstall

### macOS / Linux shell

```bash
cat scripts/uninstall.sh | LUMA_BIN_DIR=$HOME/.local/bin sh
```

### Windows PowerShell

```powershell
Get-Content .\scripts\uninstall.ps1 -Raw | Invoke-Expression
```

## Quick Start

```bash
# Analyze reclaimable storage
./target/release/luma scan

# Analyze full instance hotspots with deeper path aggregation
./target/release/luma scan --all-instances --hotspots-depth 3 --hotspots-top 60

# Compare against previous snapshot and save current snapshot
./target/release/luma scan --all-instances --hotspots-diff

# Show worlds with breakdown of large buckets
./target/release/luma worlds --breakdown

# Dry-run clean with preview filters and interactive selection
./target/release/luma clean --dry-run --kind global --min-size 200MB --older-than-days 30 --select

# Apply cleanup (moves files to trash)
./target/release/luma clean --apply -y
```

## Commands

- `luma scan`
- `luma clean`
- `luma mods`
- `luma worlds`
- `luma usage`
- `luma config`

Useful scan options:

- `--all-instances`
- `--instance <name>` (repeatable)
- `--hotspots-depth <n>` (default: `2`)
- `--hotspots-top <n>` (default: `30`)
- `--hotspots-diff` (compare with previous snapshot and update snapshot)

Useful clean options:

- `--kind <kind>` (repeatable: `global`, `instance`, `advanced`)
- `--min-size <size>` (e.g. `500MB`, `2GB`)
- `--older-than-days <days>`
- `--include-map-caches` (opt-in map tile caches: JourneyMap/Xaero/VoxelMap, nested cache-like paths included)
- `--select` (interactive candidate selection)

## Release Automation

This repository includes automated release flow in:

- [.github/workflows/release.yml](.github/workflows/release.yml)

Behavior:

1. Push to `main`
2. Workflow reads `version` from `Cargo.toml`
3. If tag `v<version>` does not exist, it creates and pushes it
4. Builds binaries for macOS (x86_64/aarch64) and Windows (x86_64)
5. Publishes GitHub Release with attached archives and `SHA256SUMS.txt`

That means your release workflow is driven by `Cargo.toml` version only.

## Safety

luma-prism is designed to avoid accidental data loss.

- Dry-run is the default cleanup mode
- Cleanup confirmation is required unless `-y` is set
- Deletions are sent to system trash, not hard-deleted
- PrismLauncher root bounds are checked before cleanup
