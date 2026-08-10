use tauri::State;

use super::{
    domain::{ChapterContent, DiscoveryList, HealthStatus, NovelOverview, RecommendBlock},
    error::NovelError,
    NovelService,
};

#[tauri::command]
pub(crate) async fn discovery_health(
    service: State<'_, NovelService>,
) -> Result<HealthStatus, NovelError> {
    service.health().await
}

#[tauri::command]
pub(crate) async fn get_recommendations(
    service: State<'_, NovelService>,
) -> Result<Vec<RecommendBlock>, NovelError> {
    service.recommend().await
}

#[tauri::command]
pub(crate) async fn get_ranking(
    service: State<'_, NovelService>,
    sort: String,
    page: u32,
) -> Result<DiscoveryList, NovelError> {
    service.ranking(sort.trim(), page).await
}

#[tauri::command]
pub(crate) async fn get_category(
    service: State<'_, NovelService>,
    tag: String,
    sort: String,
    page: u32,
) -> Result<DiscoveryList, NovelError> {
    service.category(tag.trim(), sort.trim(), page).await
}

#[tauri::command]
pub(crate) async fn search_discovery(
    service: State<'_, NovelService>,
    query: String,
    page: u32,
) -> Result<DiscoveryList, NovelError> {
    service.discovery_search(query.trim(), page).await
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
pub(crate) async fn get_novel_cover_data_url(
    service: State<'_, NovelService>,
    source: String,
    novel_id: String,
) -> Result<String, NovelError> {
    service.cover_data_url(source.trim(), &novel_id).await
}

#[tauri::command]
pub(crate) async fn get_chapter(
    service: State<'_, NovelService>,
    source: String,
    novel_id: String,
    chapter_id: String,
    chapter_title: Option<String>,
) -> Result<ChapterContent, NovelError> {
    service
        .chapter(
            source.trim(),
            &novel_id,
            &chapter_id,
            chapter_title.as_deref(),
        )
        .await
}

#[tauri::command]
pub(crate) async fn prefetch_chapters(
    service: State<'_, NovelService>,
    source: String,
    novel_id: String,
    chapter_ids: Vec<String>,
    chapter_titles: Option<Vec<String>>,
) -> Result<(), NovelError> {
    if chapter_ids.len() > 2 {
        return Err(NovelError::invalid_input(
            "at most two chapters can be prefetched",
        ));
    }
    let chapter_titles = chapter_titles.unwrap_or_default();
    if !chapter_titles.is_empty() && chapter_titles.len() != chapter_ids.len() {
        return Err(NovelError::invalid_input(
            "chapter titles must match the chapter id count",
        ));
    }
    service
        .prefetch(source.trim(), &novel_id, &chapter_ids, &chapter_titles)
        .await
}
