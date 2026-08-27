mod api;
mod commands;
mod dto;
mod error;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_movel_credentials::init())
        .plugin(tauri_plugin_dev_invoke::init())
        .setup(|app| {
            use tauri_plugin_movel_credentials::CredentialStoreExt;

            let client = api::OfficialClient::new(app.credential_store().clone())
                .expect("failed to initialize official API client");
            app.manage(client);
            app.manage(api::cache::AppCache::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::user::login,
            commands::user::register,
            commands::user::send_register_email,
            commands::user::restore_user,
            commands::user::set_avatar,
            commands::user::sign_in,
            commands::user::logout,
            commands::novel::list_novels,
            commands::novel::rank_novels,
            commands::novel::search_novels,
            commands::novel::get_reader_overview,
            commands::novel::get_reader_document,
            commands::novel::save_read_position,
            commands::bookshelf::list_bookshelf,
            commands::bookshelf::set_novel_bookshelf,
            commands::comic::list_comics,
            commands::comic::search_comics,
            commands::comic::list_comic_bookshelf,
            commands::comic::is_on_comic_bookshelf,
            commands::comic::set_comic_bookshelf,
            commands::comic::get_comic_series,
            commands::comic::get_comic_book,
            commands::comic::get_comic_chapter_pages,
            commands::comic::save_comic_read_position,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Movel");
}
