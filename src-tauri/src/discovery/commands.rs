use std::sync::Arc;

use tauri::State;

use super::domain::{DiscoveryList, HealthStatus, RecommendBlock};
use crate::error::NovelError;
use crate::sources::wenku8_api::Wenku8ApiSource;

/// 返回发现源的可用状态和登录状态。
#[tauri::command]
pub(crate) async fn discovery_health(
    source: State<'_, Arc<Wenku8ApiSource>>,
) -> Result<HealthStatus, NovelError> {
    source.health().await
}
/// 返回发现源提供的小说推荐分组。
#[tauri::command]
pub(crate) async fn get_recommendations(
    source: State<'_, Arc<Wenku8ApiSource>>,
) -> Result<Vec<RecommendBlock>, NovelError> {
    source.recommend().await
}
/// 返回按指定榜单排序的一页小说。
#[tauri::command]
pub(crate) async fn get_ranking(
    source: State<'_, Arc<Wenku8ApiSource>>,
    sort: String,
    page: u32,
) -> Result<DiscoveryList, NovelError> {
    source.ranking(sort.trim(), page).await
}
/// 返回指定分类下的一页小说。
#[tauri::command]
pub(crate) async fn get_category(
    source: State<'_, Arc<Wenku8ApiSource>>,
    tag: String,
    sort: String,
    page: u32,
) -> Result<DiscoveryList, NovelError> {
    source.category(tag.trim(), sort.trim(), page).await
}
/// 按书名搜索小说并返回一页结果。
#[tauri::command]
pub(crate) async fn search_discovery(
    source: State<'_, Arc<Wenku8ApiSource>>,
    query: String,
    page: u32,
) -> Result<DiscoveryList, NovelError> {
    source.search_mode(query.trim(), "articlename", page).await
}
