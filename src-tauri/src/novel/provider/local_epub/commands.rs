use tauri::State;

use crate::{
    library::LibraryService,
    novel::{
        domain::{ChapterContent, NovelOverview},
        error::NovelError,
    },
};

use super::LocalEpubSource;

#[tauri::command]
pub(crate) fn import_epub(
    source: State<'_, LocalEpubSource>,
    library: State<'_, LibraryService>,
    path: String,
) -> Result<NovelOverview, NovelError> {
    let overview = source.import(&path)?;
    library
        .add_book(&overview.detail)
        .map_err(|_| NovelError::Internal)?;
    Ok(overview)
}

#[tauri::command]
pub(crate) fn get_local_epub_overview(
    source: State<'_, LocalEpubSource>,
    book_id: String,
) -> Result<NovelOverview, NovelError> {
    source.overview(&book_id)
}

#[tauri::command]
pub(crate) fn get_local_epub_chapter(
    source: State<'_, LocalEpubSource>,
    book_id: String,
    chapter_id: String,
) -> Result<ChapterContent, NovelError> {
    source.chapter(&book_id, &chapter_id)
}

#[tauri::command]
pub(crate) fn get_local_epub_asset_data_url(
    source: State<'_, LocalEpubSource>,
    book_id: String,
    resource_path: String,
) -> Result<String, NovelError> {
    source.asset_data_url(&book_id, &resource_path)
}
