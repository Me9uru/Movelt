fn main() {
    const COMMANDS: &[&str] = &[
        "search_novels",
        "get_novel_overview",
        "get_chapter",
        "list_bookshelf",
        "add_to_bookshelf",
        "remove_from_bookshelf",
        "get_reading_progress",
        "save_reading_progress",
    ];

    let attributes = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS));
    tauri_build::try_build(attributes).expect("failed to run Tauri build script");
}
