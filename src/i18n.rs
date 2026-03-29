use crate::cli::Language;

#[derive(Debug, Clone, Copy)]
pub enum Msg {
    RootMissing,
    RootLabel,
    StatusDone,
    StatusFailed,
    TaskScanCleanup,
    TaskScanUnusedLibraries,
    TaskScanUnusedAssets,
    TaskScanInstanceHotspots,
    TaskCleanTargets,
    TaskScanDuplicateMods,
    TaskScanWorlds,
    TaskScanUsage,
    ConfigPromptDefaultLanguage,
    ConfigReadSelectionFailed,
    SelectInstancesPrompt,
    SelectInstancesReadFailed,
    CleanConfirmPrompt,
    CleanConfirmReadFailed,
    CleanPathOutsideRoot,
    CleanScheduled,
    CleanMovedToTrash,
    CleanFailedPrefix,
    CleanSelectPrompt,
    CleanSelectReadFailed,
    ScanTitle,
    ScanSafeTargets,
    ScanSafeTotal,
    ScanUnusedLibraries,
    ScanUnusedLibrariesTotal,
    ScanUnusedAssets,
    ScanUnusedAssetsTotal,
    ScanInstanceHotspots,
    ScanInstanceHotspotsByCategory,
    ScanInstanceHotspotsInstanceTags,
    ScanInstanceHotspotsTotal,
    ScanNone,
    PagerHelp,
    NoDuplicateMods,
    DuplicateMods,
    DuplicateGroups,
    PotentialReclaimable,
    NoWorldsDetected,
    Worlds,
    TotalWorldSize,
    InstanceUsage,
    TotalInstanceSize,
    CleanupResult,
    DryRunReclaimable,
    Cleaned,
}

pub fn text(lang: Language, msg: Msg) -> &'static str {
    match (lang, msg) {
        (Language::En, Msg::RootMissing) => "PrismLauncher root does not exist",
        (Language::Ja, Msg::RootMissing) => "PrismLauncher root が存在しません",
        (Language::En, Msg::RootLabel) => "root",
        (Language::Ja, Msg::RootLabel) => "ルート",
        (Language::En, Msg::StatusDone) => "done",
        (Language::Ja, Msg::StatusDone) => "完了",
        (Language::En, Msg::StatusFailed) => "failed",
        (Language::Ja, Msg::StatusFailed) => "失敗",
        (Language::En, Msg::TaskScanCleanup) => "Scanning cleanup targets",
        (Language::Ja, Msg::TaskScanCleanup) => "クリーン対象をスキャン中",
        (Language::En, Msg::TaskScanUnusedLibraries) => "Scanning unused libraries",
        (Language::Ja, Msg::TaskScanUnusedLibraries) => "未使用 libraries をスキャン中",
        (Language::En, Msg::TaskScanUnusedAssets) => "Scanning unused assets",
        (Language::Ja, Msg::TaskScanUnusedAssets) => "未使用 assets をスキャン中",
        (Language::En, Msg::TaskScanInstanceHotspots) => "Scanning instance storage hotspots",
        (Language::Ja, Msg::TaskScanInstanceHotspots) => {
            "インスタンス内の容量ホットスポットをスキャン中"
        }
        (Language::En, Msg::TaskCleanTargets) => "Cleaning targets",
        (Language::Ja, Msg::TaskCleanTargets) => "対象を削除中",
        (Language::En, Msg::TaskScanDuplicateMods) => "Scanning duplicate mods",
        (Language::Ja, Msg::TaskScanDuplicateMods) => "重複 mod をスキャン中",
        (Language::En, Msg::TaskScanWorlds) => "Scanning worlds",
        (Language::Ja, Msg::TaskScanWorlds) => "ワールドをスキャン中",
        (Language::En, Msg::TaskScanUsage) => "Scanning instance usage",
        (Language::Ja, Msg::TaskScanUsage) => "インスタンス使用量をスキャン中",
        (Language::En, Msg::ConfigPromptDefaultLanguage) => "Default output language",
        (Language::Ja, Msg::ConfigPromptDefaultLanguage) => "既定の出力言語",
        (Language::En, Msg::ConfigReadSelectionFailed) => "failed to read selection",
        (Language::Ja, Msg::ConfigReadSelectionFailed) => "選択の読み取りに失敗しました",
        (Language::En, Msg::SelectInstancesPrompt) => "Choose instances to scan",
        (Language::Ja, Msg::SelectInstancesPrompt) => {
            "スキャン対象のインスタンスを選択してください"
        }
        (Language::En, Msg::SelectInstancesReadFailed) => "failed to read instance selection",
        (Language::Ja, Msg::SelectInstancesReadFailed) => {
            "インスタンス選択の読み取りに失敗しました"
        }
        (Language::En, Msg::CleanConfirmPrompt) => {
            "Proceed with cleanup? (targets are moved to trash)"
        }
        (Language::Ja, Msg::CleanConfirmPrompt) => "削除を実行しますか？(対象はゴミ箱へ移動)",
        (Language::En, Msg::CleanConfirmReadFailed) => "failed to read confirmation input",
        (Language::Ja, Msg::CleanConfirmReadFailed) => "確認入力の読み取りに失敗しました",
        (Language::En, Msg::CleanPathOutsideRoot) => "path is outside PrismLauncher root",
        (Language::Ja, Msg::CleanPathOutsideRoot) => "root外のパスは処理不可",
        (Language::En, Msg::CleanScheduled) => "scheduled for deletion",
        (Language::Ja, Msg::CleanScheduled) => "削除予定",
        (Language::En, Msg::CleanMovedToTrash) => "moved to trash",
        (Language::Ja, Msg::CleanMovedToTrash) => "ゴミ箱へ移動",
        (Language::En, Msg::CleanFailedPrefix) => "failed",
        (Language::Ja, Msg::CleanFailedPrefix) => "削除失敗",
        (Language::En, Msg::CleanSelectPrompt) => "Select cleanup targets",
        (Language::Ja, Msg::CleanSelectPrompt) => "削除対象を選択してください",
        (Language::En, Msg::CleanSelectReadFailed) => "failed to read cleanup selection",
        (Language::Ja, Msg::CleanSelectReadFailed) => "削除対象の選択読み取りに失敗しました",
        (Language::En, Msg::ScanTitle) => "PrismLauncher Scan Report",
        (Language::Ja, Msg::ScanTitle) => "PrismLauncher スキャンレポート",
        (Language::En, Msg::ScanSafeTargets) => "[Safe cleanup targets]",
        (Language::Ja, Msg::ScanSafeTargets) => "[安全に削除できる対象]",
        (Language::En, Msg::ScanSafeTotal) => "Safe reclaimable total",
        (Language::Ja, Msg::ScanSafeTotal) => "安全対象の合計",
        (Language::En, Msg::ScanUnusedLibraries) => "[Potentially unused libraries]",
        (Language::Ja, Msg::ScanUnusedLibraries) => "[未使用の可能性がある libraries]",
        (Language::En, Msg::ScanUnusedLibrariesTotal) => "Unused libraries total",
        (Language::Ja, Msg::ScanUnusedLibrariesTotal) => "未使用 libraries の合計",
        (Language::En, Msg::ScanUnusedAssets) => "[Potentially unused assets]",
        (Language::Ja, Msg::ScanUnusedAssets) => "[未使用の可能性がある assets]",
        (Language::En, Msg::ScanUnusedAssetsTotal) => "Unused assets total",
        (Language::Ja, Msg::ScanUnusedAssetsTotal) => "未使用 assets の合計",
        (Language::En, Msg::ScanInstanceHotspots) => "[Instance storage hotspots (all data)]",
        (Language::Ja, Msg::ScanInstanceHotspots) => {
            "[インスタンス内の容量ホットスポット(全データ)]"
        }
        (Language::En, Msg::ScanInstanceHotspotsByCategory) => "Category summary:",
        (Language::Ja, Msg::ScanInstanceHotspotsByCategory) => "分類サマリー:",
        (Language::En, Msg::ScanInstanceHotspotsInstanceTags) => "Category tags:",
        (Language::Ja, Msg::ScanInstanceHotspotsInstanceTags) => "分類タグ:",
        (Language::En, Msg::ScanInstanceHotspotsTotal) => "Instance hotspot total",
        (Language::Ja, Msg::ScanInstanceHotspotsTotal) => "ホットスポット合計",
        (Language::En, Msg::ScanNone) => "(none)",
        (Language::Ja, Msg::ScanNone) => "(候補なし)",
        (Language::En, Msg::PagerHelp) => {
            "Page {page}/{total}  Next: Right/j/l  Prev: Left/h/k  Quit: q/Enter"
        }
        (Language::Ja, Msg::PagerHelp) => {
            "ページ {page}/{total}  次へ: Right/j/l  前へ: Left/h/k  終了: q/Enter"
        }
        (Language::En, Msg::NoDuplicateMods) => "No duplicate mods found.",
        (Language::Ja, Msg::NoDuplicateMods) => "重複 mod は見つかりませんでした。",
        (Language::En, Msg::DuplicateMods) => "Duplicate mods:",
        (Language::Ja, Msg::DuplicateMods) => "重複 mod 一覧:",
        (Language::En, Msg::DuplicateGroups) => "Duplicate groups",
        (Language::Ja, Msg::DuplicateGroups) => "重複グループ数",
        (Language::En, Msg::PotentialReclaimable) => "Potential reclaimable",
        (Language::Ja, Msg::PotentialReclaimable) => "削減可能見込み",
        (Language::En, Msg::NoWorldsDetected) => "No worlds detected in selected instances.",
        (Language::Ja, Msg::NoWorldsDetected) => {
            "選択されたインスタンスでワールドは検出されませんでした。"
        }
        (Language::En, Msg::Worlds) => "Worlds:",
        (Language::Ja, Msg::Worlds) => "ワールド一覧:",
        (Language::En, Msg::TotalWorldSize) => "Total world size",
        (Language::Ja, Msg::TotalWorldSize) => "ワールド合計サイズ",
        (Language::En, Msg::InstanceUsage) => "Instance usage:",
        (Language::Ja, Msg::InstanceUsage) => "インスタンス使用量:",
        (Language::En, Msg::TotalInstanceSize) => "Total instance size",
        (Language::Ja, Msg::TotalInstanceSize) => "インスタンス合計サイズ",
        (Language::En, Msg::CleanupResult) => "Cleanup result:",
        (Language::Ja, Msg::CleanupResult) => "削除結果:",
        (Language::En, Msg::DryRunReclaimable) => "Dry-run reclaimable",
        (Language::Ja, Msg::DryRunReclaimable) => "dry-run で削減可能",
        (Language::En, Msg::Cleaned) => "Cleaned",
        (Language::Ja, Msg::Cleaned) => "削除済み",
    }
}
