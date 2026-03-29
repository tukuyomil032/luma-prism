# Scan Coverage Enhancement

## 目的

Mod 由来ディレクトリを含む `instances/*/.minecraft` 全体を見える化し、
「既知キャッシュだけが見える」状態から「未知 Mod の肥大化も検出できる」状態へ改善する。

## Top500 調査 (Modrinth)

調査日: 2026-03-29

### 収集条件

- API: `https://api.modrinth.com/v2/search`
- フィルタ: `project_type:mod`
- 並び: `index=downloads`
- 件数: `limit=100` x `offset=0,100,200,300,400` = 500 件

### 集計結果

- Top500 総数: 500
- `source_url` 設定あり: 465
- GitHub `source_url`: 449
- 非GitHub `source_url`: 16 (GitLab / Codeberg など)
- `source_url` 未設定: 35

### 重要な制約

- Top500 全件について「各 Mod の全ソースを均等深度で精読」は現実的ではない
  - 一部はソース非公開または `source_url` 未設定
  - 一部は GitHub 以外のホスト
  - したがって、スキャナ設計は個別 Mod ハードコードより「未知ディレクトリ検出」に寄せる必要がある

## 上位 Mod と保存構造の実コード確認

Top500 内の高DL Mod から、保存先が容量増加に直結するものを優先確認した。

### JourneyMap

- `TeamJM/journeymap` は issue tracker 用公開リポジトリ
- README に「Source code is not available to the public (ARR)」と明記
- 保存構造は公開ソースで追跡不可

### Essential (SparkUniverse/Essential-Mod)

- ベース: `.minecraft/essential`
- スクショ本体: `.minecraft/screenshots`
- キャッシュ:
  - `.minecraft/essential/screenshot-cache`
  - `.minecraft/essential/screenshot-checksum-caches.json`
  - `.minecraft/essential/cosmetic-cache`

### ReplayMod (ReplayMod/ReplayMod)

- 録画: `.minecraft/replay_recordings`
- キャッシュ: `.minecraft/.replay_cache`
- 動画出力: `.minecraft/replay_videos`
- スクショ: `.minecraft/screenshots`

### Litematica (maruohon/litematica)

- `.minecraft/schematics`
- `.minecraft/litematica/world_specific_data`
- `.minecraft/litematica/placements`
- `config/litematica/material_cache.nbt`

### Iris / Sodium / FTB Chunks

- Iris: `.minecraft/shaderpacks`, `config/iris.properties`
- Sodium: `config/sodium-options.json`
- FTB Chunks: `<world>/ftbchunks`, `local/ftbchunks`

## 既存スキャンの穴

旧設計では、以下のギャップがあった。

1. 安全削除候補中心で、実容量の上位パスを見失う
2. Mod 固有ディレクトリ (例: `journeymap`, `essential`) の増加を常時可視化できない
3. ワールド直下の Mod 生成データ (`saves/<world>/...`) の相対比較が弱い

## 今回の対処

### 実装済み

1. `scan` にインスタンスホットスポット解析を統合
- `.minecraft` 配下を全ファイル走査
- プレフィックス集計 (`depth` 指定)
- JSON に `instance_hotspots` を追加

2. 穴対策として、depth1 を常時表示
- `hotspots_top` は「入れ子パス上位 N」にのみ適用
- これにより、トップレベルのディレクトリ種別は取りこぼさない

3. CLI パラメータ化
- `luma scan --hotspots-depth <n> --hotspots-top <n>`
- 既定値: `depth=2`, `top=30`

4. ホットスポットに分類タグを付与
- 出力カテゴリ: `world`, `media`, `map-data`, `mod-cache`, `logs`, `resource`, `mods`, `config`, `unknown`
- Top500 調査で確認した代表パス (Essential/Replay/Litematica/FTB Chunks/JourneyMap 系) を優先分類
- インスタンス単位 / 全体単位のカテゴリ合計を表示

### 設計意図

Top500 全件の個別知識を追加し続ける方式は破綻しやすいため、
「未知 Mod でも見える」汎用走査を主系に置き、
代表的 Mod の既知パスは解釈補助として扱う。

## 推奨する次段階

1. 差分スナップショット
- 前回比で増加したパスを強調

2. 安全削除候補の拡張は段階的に
- 既知の再生成可能キャッシュのみ `clean` 候補へ昇格
- `saves`, `mods`, `config`, `resourcepacks` は引き続き保護
