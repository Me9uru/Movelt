mod api;
mod commands;
mod dto;
mod error;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let client = api::OfficialClient::new().expect("failed to initialize official API client");
    tauri::Builder::default()
        .plugin(tauri_plugin_dev_invoke::init())
        .manage(client)
        .invoke_handler(tauri::generate_handler![
            commands::user::login,
            commands::user::register,
            commands::user::send_register_email,
            commands::user::restore_user,
            commands::user::logout,
            commands::novel::get_latest,
            commands::novel::get_ranking,
            commands::novel::get_rank,
            commands::novel::search_novels,
            commands::novel::get_reader_overview,
            commands::novel::get_reader_document,
            commands::novel::save_read_position,
            commands::bookshelf::list_bookshelf,
            commands::bookshelf::set_novel_bookshelf,
            commands::manga::browse_manga,
            commands::manga::list_manga_bookshelf,
            commands::manga::is_on_manga_bookshelf,
            commands::manga::set_manga_bookshelf,
            commands::manga::get_manga,
            commands::manga::get_manga_chapter_pages,
            commands::manga::get_manga_page_batch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Movel");
}
