use std::sync::Arc;

use tauri::State;

use crate::{
    error::NovelError,
    sources::lnovel_api::{
        LnovelApiSource, MangaDetail, MangaPageBatch, MangaPageList, MangaSummary,
    },
};

#[tauri::command]
pub(crate) async fn browse_manga(
    source: State<'_, Arc<LnovelApiSource>>,
    query: Option<String>,
    page: i32,
    browse_type: String,
) -> Result<Vec<MangaSummary>, NovelError> {
    source.browse(query.as_deref(), page, &browse_type).await
}

#[tauri::command]
pub(crate) async fn get_manga(
    source: State<'_, Arc<LnovelApiSource>>,
    manga_id: String,
) -> Result<MangaDetail, NovelError> {
    source.manga(&manga_id).await
}

#[tauri::command]
pub(crate) async fn get_manga_chapter_pages(
    source: State<'_, Arc<LnovelApiSource>>,
    manga_id: String,
    chapter_id: String,
) -> Result<MangaPageList, NovelError> {
    source.chapter_pages(&manga_id, &chapter_id).await
}

#[tauri::command]
pub(crate) async fn get_manga_page_batch(
    source: State<'_, Arc<LnovelApiSource>>,
    chapter_id: String,
    page_index: usize,
) -> Result<MangaPageBatch, NovelError> {
    source.page_batch(&chapter_id, page_index).await
}
