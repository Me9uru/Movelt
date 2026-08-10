fn main() {
    const COMMANDS: &[&str] = &[
        "discovery_health",
        "get_recommendations",
        "get_ranking",
        "get_category",
        "search_discovery",
        "get_novel_overview",
        "get_novel_cover_data_url",
        "get_chapter",
        "prefetch_chapters",
        "import_epub",
        "get_local_epub_overview",
        "get_local_epub_chapter",
        "get_local_epub_asset_data_url",
        "list_bookshelf",
        "search_bookshelf",
        "add_to_bookshelf",
        "remove_from_bookshelf",
        "get_reading_progress",
        "save_reading_progress",
    ];

    let attributes = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS));
    tauri_build::try_build(attributes).expect("failed to run Tauri build script");
}
