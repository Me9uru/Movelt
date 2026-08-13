use std::sync::Arc;

use tauri::State;

use crate::{domain::NovelOverview, error::NovelError, library::LibraryService};

use super::LocalEpubSource;

/// 导入本地 EPUB 并将其加入书架。
#[tauri::command]
pub(crate) fn import_epub(
    source: State<'_, Arc<LocalEpubSource>>,
    library: State<'_, LibraryService>,
    path: String,
) -> Result<NovelOverview, NovelError> {
    let overview = source.import(&path)?;
    library
        .add_book(&overview.detail)
        .map_err(|_| NovelError::Internal)?;
    Ok(overview)
}

/// 将 EPUB 资源加载为可显示的数据 URL。
#[tauri::command]
pub(crate) fn get_local_epub_asset_data_url(
    source: State<'_, Arc<LocalEpubSource>>,
    book_id: String,
    resource_path: String,
) -> Result<String, NovelError> {
    source.asset_data_url(&book_id, &resource_path)
}
