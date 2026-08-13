use std::sync::Arc;

use tauri::State;

use super::local_epub::LocalEpubSource;
use super::{
    domain::{BookshelfEntry, ReadingProgress, ReadingProgressInput},
    LibraryService,
};
use crate::{domain::NovelDetail, error::LibraryError};

/// 返回书架中保存的全部书籍。
#[tauri::command]
pub(crate) fn list_bookshelf(
    service: State<'_, LibraryService>,
) -> Result<Vec<BookshelfEntry>, LibraryError> {
    service.list_books()
}

/// 按书名搜索书架中的书籍。
#[tauri::command]
pub(crate) fn search_bookshelf(
    service: State<'_, LibraryService>,
    query: String,
) -> Result<Vec<BookshelfEntry>, LibraryError> {
    service.search_books(&query)
}

/// 向书架添加或更新一本书。
#[tauri::command]
pub(crate) fn add_to_bookshelf(
    service: State<'_, LibraryService>,
    book: NovelDetail,
) -> Result<(), LibraryError> {
    service.add_book(&book)
}

/// 从书架移除书籍及其关联的本地 EPUB 数据。
#[tauri::command]
pub(crate) fn remove_from_bookshelf(
    service: State<'_, LibraryService>,
    local_epub: State<'_, Arc<LocalEpubSource>>,
    source: String,
    book_id: String,
) -> Result<(), LibraryError> {
    service.remove_book(&source, &book_id)?;
    if source == LocalEpubSource::SOURCE_ID {
        local_epub
            .remove(&book_id)
            .map_err(|error| LibraryError::Database(error.to_string()))?;
    }
    Ok(())
}

/// 返回一本书已保存的阅读进度。
#[tauri::command]
pub(crate) fn get_reading_progress(
    service: State<'_, LibraryService>,
    source: String,
    book_id: String,
) -> Result<Option<ReadingProgress>, LibraryError> {
    service.get_progress(&source, &book_id)
}

/// 校验并保存一本书的当前阅读进度。
#[tauri::command]
pub(crate) fn save_reading_progress(
    service: State<'_, LibraryService>,
    source: String,
    book_id: String,
    progress: ReadingProgressInput,
) -> Result<ReadingProgress, LibraryError> {
    service.save_progress(&source, &book_id, &progress)
}
