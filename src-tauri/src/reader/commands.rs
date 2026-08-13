use super::{service::SourceId, ReaderService};
use crate::error::NovelError;
use crate::{domain::NovelOverview, reader::domain::ReaderDocument};
use tauri::State;

/// 加载打开书籍所需的元数据和文档目录。
#[tauri::command]
pub(crate) async fn get_reader_overview(
    service: State<'_, ReaderService>,
    source: String,
    book_id: String,
) -> Result<NovelOverview, NovelError> {
    service.overview(SourceId::parse(&source)?, &book_id).await
}

/// 将书籍封面加载为供阅读器显示的数据 URL。
#[tauri::command]
pub(crate) async fn get_reader_cover_data_url(
    service: State<'_, ReaderService>,
    source: String,
    book_id: String,
) -> Result<String, NovelError> {
    service
        .cover_data_url(SourceId::parse(&source)?, &book_id)
        .await
}

/// 加载一个文档并转换为统一的阅读器格式。
#[tauri::command]
pub(crate) async fn get_reader_document(
    service: State<'_, ReaderService>,
    source: String,
    book_id: String,
    document_id: String,
    document_title: Option<String>,
) -> Result<ReaderDocument, NovelError> {
    let chapter = service
        .chapter(
            SourceId::parse(&source)?,
            &book_id,
            &document_id,
            document_title.as_deref(),
        )
        .await?;
    Ok(chapter.into())
}
