mod library;
mod novel;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let novel_state = novel::NovelState::new().expect("failed to initialize novel source");

    tauri::Builder::default()
        .plugin(tauri_plugin_dev_invoke::init())
        .manage(novel_state)
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            app.manage(library::LibraryState::new(
                &data_dir.join("library.sqlite3"),
            )?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            novel::search_novels,
            novel::get_novel_overview,
            novel::get_chapter,
            library::list_bookshelf,
            library::add_to_bookshelf,
            library::remove_from_bookshelf,
            library::get_reading_progress,
            library::save_reading_progress
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
