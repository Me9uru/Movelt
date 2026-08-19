mod discovery;
mod domain;
mod error;
mod library;
mod manga;
mod reader;
mod sources;

use std::sync::Arc;

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
            let novel_config = discovery::NovelConfig::load()?;
            let wenku8_api = Arc::new(sources::wenku8_api::Wenku8ApiSource::new(
                &novel_config.wenku8_api_base_url,
            )?);
            app.manage(Arc::clone(&wenku8_api));
            app.manage(Arc::new(sources::lnovel_api::LnovelApiSource::new(
                &novel_config.lnovel_api_base_url,
            )?));
            let local_epub = Arc::new(library::local_epub::LocalEpubSource::new(epub_root)?);
            app.manage(reader::ReaderService::new(
                wenku8_api,
                Arc::clone(&local_epub),
            ));
            app.manage(local_epub);
            app.manage(library::LibraryService::open(
                &data_dir.join("library.sqlite3"),
            )?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            discovery::commands::discovery_health,
            discovery::commands::get_recommendations,
            discovery::commands::get_ranking,
            discovery::commands::get_category,
            discovery::commands::search_discovery,
            reader::commands::get_reader_overview,
            reader::commands::get_reader_cover_data_url,
            reader::commands::get_reader_document,
            library::local_epub::commands::import_epub,
            library::local_epub::commands::get_local_epub_asset_data_url,
            library::commands::list_bookshelf,
            library::commands::search_bookshelf,
            library::commands::add_to_bookshelf,
            library::commands::remove_from_bookshelf,
            library::commands::get_reading_progress,
            library::commands::save_reading_progress,
            manga::browse_manga,
            manga::get_manga,
            manga::get_manga_chapter_pages,
            manga::get_manga_page_batch
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
