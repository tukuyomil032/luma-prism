# AGENTS.md

## 1. Purpose

This document provides instructions specifically for AI coding agents (e.g., Devin, Cursor, Copilot agents).

Before performing any work in this repository, the agent MUST read this file to understand:

- project architecture
- development rules
- safety constraints
- filesystem assumptions
- workflow expectations

The goal is to ensure agents make safe and correct changes to the project.

This document is written to be readable by both humans and AI agents.

---

# 2. Project Overview

**Project Name**

luma

**Description**

luma is a high-performance Rust CLI tool that analyzes and optimizes PrismLauncher storage usage.

It scans the PrismLauncher filesystem and identifies unnecessary or duplicate data.

### Primary Features

| Feature | Description |
|------|-------------|
| Storage Scan | Analyze disk usage |
| Cache Cleanup | Remove launcher cache |
| Log Cleanup | Remove logs |
| Unused Library Detection | Detect unused Minecraft libraries |
| Duplicate Mod Detection | Find identical mods across instances |
| World Size Analyzer | Detect large worlds |
| Asset Deduplication | Remove unused assets |
| Modpack Cache Cleanup | Remove cached downloads |

luma must **never modify PrismLauncher configuration files**.

It only analyzes and cleans filesystem data.

---

# 3. Supported Platforms

| OS | Support |
|----|--------|
| Windows | Supported |
| macOS | Supported |

Linux support may be added later.

---

# 4. PrismLauncher Filesystem Layout

luma operates on the PrismLauncher data directory.

### Windows

```
%APPDATA%/PrismLauncher
```

### macOS

```
~/Library/Application Support/PrismLauncher
```

### Important Directories

| Directory | Description |
|-----------|-------------|
| instances/ | Minecraft instances |
| libraries/ | Minecraft runtime libraries |
| assets/ | Minecraft asset storage |
| cache/ | Download cache |
| meta/ | Metadata cache |
| logs/ | Launcher logs |
| java/ | Managed Java installations |
| icons/ | Instance icons |

Agents must **never assume directories outside this root**.

---

# 5. Instance Structure

Instances exist under:

```
instances/<instance_name>/
```

Important files:

```
instance.cfg
mmc-pack.json
.minecraft/
```

Inside `.minecraft`:

```
mods/
config/
saves/
logs/
resourcepacks/
shaderpacks/
crash-reports/
```

### Protected Directories

The following directories must **never be deleted automatically**:

| Directory | Reason |
|----------|--------|
| mods | Installed mods |
| config | Mod configs |
| saves | World saves |
| resourcepacks | User assets |

---

# 6. Safe Cleanup Targets

The following directories are safe to clean:

| Path | Description |
|-----|-------------|
| cache/ | Download cache |
| logs/ | Launcher logs |
| meta/ | Metadata cache |
| instances/*/.minecraft/logs | Game logs |
| instances/*/.minecraft/crash-reports | Crash reports |

---

# 7. Duplicate Mod Detection

Goal: detect identical mods across instances.

Algorithm:

1. scan all `mods/` directories
2. hash each `.jar`
3. group identical hashes
4. report duplicates

Example:

```
Mod: Sodium.jar

Instances:
FabricPack
VanillaPlus
TechPack
```

---

# 8. World Size Analyzer

Scan:

```
instances/*/.minecraft/saves
```

Compute directory sizes.

Example output:

```
Instance: TechPack
World: survival_world
Size: 3.4GB
```

---

# 9. Asset Deduplication

Assets stored in:

```
assets/objects/
```

Algorithm:

1. read asset index files
2. collect referenced hashes
3. detect orphan files
4. mark unused assets

Agents must verify asset usage before deletion.

---

# 10. CLI Design

Main command:

```
luma
```

### Subcommands

| Command | Description |
|--------|-------------|
| scan | analyze disk usage |
| clean | remove unnecessary files |
| mods | analyze duplicate mods |
| worlds | analyze world sizes |
| usage | show instance usage |

Example usage:

```
luma scan
luma clean --dry-run
luma mods
luma worlds
```

---

# 11. CLI User Experience

luma should provide clear terminal feedback.

Example spinner output:

```
Scanning instances...
⠋ hashing mods
⠋ scanning worlds
⠋ scanning libraries
```

Recommended crates:

| Crate | Purpose |
|------|---------|
| indicatif | spinners / progress bars |
| console | terminal styling |
| colored | colored text |
| tabled | table output |

---

# 12. Performance Requirements

Large installations may contain:

- 300k+ files
- 50GB+ data

Use parallel scanning.

Recommended crates:

| Crate | Usage |
|------|------|
| rayon | parallel processing |
| walkdir | filesystem traversal |

Parallel tasks:

- instance scanning
- mod hashing
- library scanning
- world size calculation

---

# 13. Safety Rules

luma must never cause data loss.

Mandatory safety features:

| Rule | Description |
|-----|-------------|
| dry-run | default mode |
| confirmation | required before deletion |
| trash support | move to system trash |

Rust crate:

```
trash
```

---

# 14. Project Structure

```
src/

main.rs

cli/
  commands.rs
  args.rs

scanner/
  instances.rs
  mods.rs
  libraries.rs
  assets.rs
  worlds.rs

analysis/
  duplicates.rs
  sizes.rs

cleaner/
  delete.rs

ui/
  spinners.rs
  tables.rs
```

---

# 15. Development Workflow

1. create feature branch from `main`
2. implement feature
3. update documentation
4. open Draft PR

Agents must **never push directly to main**.

---

# 16. Pull Request Rules

PR title format:

```
[luma] <feature description>
```

Example:

```
[luma] add duplicate mod detection
```

Requirements:

- must be Draft PR
- assign repository owner
- do not mark ready for review automatically

Agents must wait for instructions.

---

# 17. Code Style

Language: Rust

Requirements:

- Rust 2021 edition
- idiomatic Rust
- avoid unsafe
- modular architecture

Tools:

```
cargo fmt
cargo clippy
```

---

# 18. Testing

Tests should cover:

- duplicate mod detection
- world size calculation
- library scanning

Run tests with:

```
cargo test
```

Target coverage:

```
>80%
```

---

# 19. Related Agent Config Files

Agents may also reference:

```
.rules
.cursorrules
.windsurf
.mdc
```

### General
- Commit messages must be in English and use the prefixes `feat:`, `fix:`, `refactor:`, or `docs:`.
- After completing an implementation, edit, or a single prompt task, check for any remaining items within the current phase. If all items in the current phase have been completed, check if there are items to be implemented in the next phase. If so, ask the user in question mode to select where to proceed with implementation. Do this after every implementation until all implementations in every phase are complete—that is, until the project is ready for release. Additionally, after that question, generate a command that includes a commit message appropriate for the changes made in that implementation, and ask the user in question mode whether to actually execute that command. Do not ask about generating a commit command for every change or implementation within the same phase. However, an exception is made for changes or implementations within the same phase if the scope of each individual implementation is large. Furthermore, even if the user chooses not to execute the commit command, if there are still tasks to be implemented within the same phase or if the next phase remains, continue implementation and repeat the cycle of “question → implementation → question” indefinitely until implementation is complete. The “question” referred to here is not a question asked in the form of text output by you, but rather the multiple-choice questions used when defining requirements in Plan mode. The format for the commit command should be `git commit -m “message” -m ‘message’ -m “message”`, summarizing the implemented content broadly and using the -m option to separate each line with a newline.
  - Perform Git operations by entering commands directly in the terminal within VSCode. Do not use MCP or similar tools; enter the commands directly.
    - Adding files: git add .
    - Commit: As described in the bullet point above, use the format that includes a commit message appropriate for the changes made in each implementation. `git commit -m “message” -m ‘message’ -m “message”`
    - Push: `git push origin main`
      - If you run a reset command such as `git reset --soft HEAD^` due to some issue, I will notify you. In that case, you must perform a force push (`git push -f -u origin main`) to avoid conflicts and rejection.
- After completing the implementation or editing for the given prompt, before building, read all files (.java, .ts, .json, .yml, etc.) from the background. If there are any errors or warnings, investigate the details and cause of the errors and fix them. Since a single fix may not resolve the issue, retrieve the errors from the background again after making the fix and continue the process until there are zero errors and warnings.
- If there are multiple updates or additions, number them sequentially (1, 2, 3) and assign them to separate tasks. Additionally, all tasks with these numbers must be implemented in a single prompt. Proceed with implementation step by step, reporting the progress of each step to the user as you go.
  - For example, if there are three tasks—1. Converting to binary IPC (removing Base64), 2. Adaptive FPS control, and 3. Extending rectangular difference rendering on the Java side—and you are instructed to “complete tasks 1 through 3,” you must finish implementing all of them in a single round of prompts.


AGENTS.md remains the primary instruction source.
