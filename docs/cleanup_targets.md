# Cleanup Targets

## Safe Targets

- `cache`
- `logs`
- `meta`
- `catpacks`
- `instances/*/.minecraft/logs`
- `instances/*/.minecraft/crash-reports`
- `instances/*/.minecraft/.replay_cache`
- `instances/*/.minecraft/essential/screenshot-cache`
- `instances/*/.minecraft/essential/cosmetic-cache`
- `instances/*/.minecraft/essential/screenshot-checksum-caches.json`

## Conditional Targets

- unused libraries (`--include-unused-libraries`)
- unused assets (`--include-unused-assets`)
- map tile caches (`--include-map-caches`)
	- `instances/*/.minecraft/journeymap/cache`
	- `instances/*/.minecraft/journeymap/webmap`
	- `instances/*/.minecraft/xaerominimap/cache`
	- `instances/*/.minecraft/xaeroworldmap/cache`
	- `instances/*/.minecraft/voxelmap/cache`

## Protected (Never Auto-delete)

- `instances/*/.minecraft/mods`
- `instances/*/.minecraft/config`
- `instances/*/.minecraft/saves`
- `instances/*/.minecraft/resourcepacks`
