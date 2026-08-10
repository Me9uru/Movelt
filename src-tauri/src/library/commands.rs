use tauri::State;

use super::{
    domain::{BookshelfEntry, ReadingProgress, ReadingProgressInput},
    error::LibraryError,
    LibraryService,
};
use crate::novel::domain::NovelDetail;
use crate::novel::provider::local_epub::LocalEpubSource;

#[tauri::command]
pub(crate) fn list_bookshelf(
    service: State<'_, LibraryService>,
) -> Result<Vec<BookshelfEntry>, LibraryError> {
    service.list_books()
}

#[tauri::command]
pub(crate) fn search_bookshelf(
    service: State<'_, LibraryService>,
    query: String,
) -> Result<Vec<BookshelfEntry>, LibraryError> {
    service.search_books(&query)
}

#[tauri::command]
pub(crate) fn add_to_bookshelf(
    service: State<'_, LibraryService>,
    book: NovelDetail,
) -> Result<(), LibraryError> {
    service.add_book(&book)
}

#[tauri::command]
pub(crate) fn remove_from_bookshelf(
    service: State<'_, LibraryService>,
    local_epub: State<'_, LocalEpubSource>,
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

#[tauri::command]
pub(crate) fn get_reading_progress(
    service: State<'_, LibraryService>,
    source: String,
    book_id: String,
) -> Result<Option<ReadingProgress>, LibraryError> {
    service.get_progress(&source, &book_id)
}

#[tauri::command]
pub(crate) fn save_reading_progress(
    service: State<'_, LibraryService>,
    source: String,
    book_id: String,
    progress: ReadingProgressInput,
) -> Result<ReadingProgress, LibraryError> {
    service.save_progress(&source, &book_id, &progress)
}
