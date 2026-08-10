mod library;
mod novel;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dev_invoke::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let epub_root = data_dir.join("local_epub");
            let novel_config = novel::NovelConfig::load()?;
            app.manage(novel::NovelService::new(&novel_config.wenku8_api_base_url)?);
            app.manage(novel::provider::local_epub::LocalEpubSource::new(
                epub_root,
            )?);
            app.manage(library::LibraryService::open(
                &data_dir.join("library.sqlite3"),
            )?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            novel::commands::discovery_health,
            novel::commands::get_recommendations,
            novel::commands::get_ranking,
            novel::commands::get_category,
            novel::commands::search_discovery,
            novel::commands::get_novel_overview,
            novel::commands::get_novel_cover_data_url,
            novel::commands::get_chapter,
            novel::commands::prefetch_chapters,
            novel::provider::local_epub::commands::import_epub,
            novel::provider::local_epub::commands::get_local_epub_overview,
            novel::provider::local_epub::commands::get_local_epub_chapter,
            novel::provider::local_epub::commands::get_local_epub_asset_data_url,
            library::commands::list_bookshelf,
            library::commands::search_bookshelf,
            library::commands::add_to_bookshelf,
            library::commands::remove_from_bookshelf,
            library::commands::get_reading_progress,
            library::commands::save_reading_progress
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
