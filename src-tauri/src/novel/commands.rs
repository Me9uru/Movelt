use tauri::State;

use super::{
    domain::{ChapterContent, NovelOverview, NovelSourceInfo, SearchResult},
    error::NovelError,
    NovelService,
};

#[tauri::command]
pub(crate) fn list_novel_sources(service: State<'_, NovelService>) -> Vec<NovelSourceInfo> {
    service.sources()
}

#[tauri::command]
pub(crate) async fn search_novels(
    service: State<'_, NovelService>,
    source: String,
    query: String,
    page: Option<u32>,
) -> Result<SearchResult, NovelError> {
    let query = query.trim();
    if query.is_empty() || query.chars().count() > 100 {
        return Err(NovelError::invalid_input(
            "query must contain between 1 and 100 characters",
        ));
    }
    let page = page.unwrap_or(1);
    if page == 0 || page > 1_000 {
        return Err(NovelError::invalid_input("page must be between 1 and 1000"));
    }
    service.search(source.trim(), query, page).await
}

#[tauri::command]
pub(crate) async fn get_novel_overview(
    service: State<'_, NovelService>,
    source: String,
    novel_id: String,
) -> Result<NovelOverview, NovelError> {
    service.overview(source.trim(), &novel_id).await
}

#[tauri::command]
pub(crate) async fn get_chapter(
    service: State<'_, NovelService>,
    source: String,
    novel_id: String,
    chapter_id: String,
) -> Result<ChapterContent, NovelError> {
    service.chapter(source.trim(), &novel_id, &chapter_id).await
}

#[tauri::command]
pub(crate) async fn prefetch_chapters(
    service: State<'_, NovelService>,
    source: String,
    novel_id: String,
    chapter_ids: Vec<String>,
) -> Result<(), NovelError> {
    if chapter_ids.len() > 2 {
        return Err(NovelError::invalid_input(
            "at most two chapters can be prefetched",
        ));
    }
    service
        .prefetch(source.trim(), &novel_id, &chapter_ids)
        .await
}
