mod library;
mod novel;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let novel_service = novel::NovelService::new().expect("failed to initialize novel source");

    tauri::Builder::default()
        .plugin(tauri_plugin_dev_invoke::init())
        .manage(novel_service)
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            app.manage(library::LibraryService::open(
                &data_dir.join("library.sqlite3"),
            )?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            novel::commands::list_novel_sources,
            novel::commands::search_novels,
            novel::commands::get_novel_overview,
            novel::commands::get_chapter,
            novel::commands::prefetch_chapters,
            library::commands::list_bookshelf,
            library::commands::add_to_bookshelf,
            library::commands::remove_from_bookshelf,
            library::commands::get_reading_progress,
            library::commands::save_reading_progress
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
