use std::path::Path;

use super::{
    domain::{BookshelfEntry, ReadingProgress, ReadingProgressInput},
    sqlite::SqliteLibraryRepository,
};
use crate::{domain::NovelDetail, error::LibraryError};

pub(crate) struct LibraryService {
    repository: SqliteLibraryRepository,
}

impl LibraryService {
    pub(crate) fn open(path: &Path) -> Result<Self, LibraryError> {
        Ok(Self {
            repository: SqliteLibraryRepository::open(path)?,
        })
    }

    #[cfg(test)]
    fn in_memory() -> Result<Self, LibraryError> {
        Ok(Self {
            repository: SqliteLibraryRepository::in_memory()?,
        })
    }

    pub(crate) fn add_book(&self, book: &NovelDetail) -> Result<(), LibraryError> {
        validate_key(&book.source, &book.id)?;
        self.repository.add_book(book)
    }

    pub(super) fn remove_book(&self, source: &str, book_id: &str) -> Result<(), LibraryError> {
        validate_key(source, book_id)?;
        self.repository.remove_book(source, book_id)
    }

    pub(super) fn list_books(&self) -> Result<Vec<BookshelfEntry>, LibraryError> {
        self.repository.list_books()
    }

    pub(super) fn search_books(&self, query: &str) -> Result<Vec<BookshelfEntry>, LibraryError> {
        let query = query.trim();
        if query.is_empty() {
            return self.list_books();
        }
        if query.chars().count() > 100 {
            return Err(LibraryError::InvalidInput(
                "bookshelf search query must not exceed 100 characters".into(),
            ));
        }
        self.repository.search_books(query)
    }

    pub(super) fn save_progress(
        &self,
        source: &str,
        book_id: &str,
        progress: &ReadingProgressInput,
    ) -> Result<ReadingProgress, LibraryError> {
        validate_key(source, book_id)?;
        if progress.document_id.trim().is_empty() || progress.document_title.trim().is_empty() {
            return Err(LibraryError::InvalidInput(
                "document id and title must not be empty".into(),
            ));
        }
        if !progress.location.is_finite() || !progress.book_location.is_finite() {
            return Err(LibraryError::InvalidInput(
                "reading locations must be finite numbers".into(),
            ));
        }
        self.repository.save_progress(
            source,
            book_id,
            progress,
            progress.location.clamp(0.0, 1.0),
            progress.book_location.clamp(0.0, 1.0),
        )
    }

    pub(super) fn get_progress(
        &self,
        source: &str,
        book_id: &str,
    ) -> Result<Option<ReadingProgress>, LibraryError> {
        validate_key(source, book_id)?;
        self.repository.get_progress(source, book_id)
    }
}

fn validate_key(source: &str, book_id: &str) -> Result<(), LibraryError> {
    if source.trim().is_empty() || book_id.trim().is_empty() {
        return Err(LibraryError::InvalidInput(
            "source and book id must not be empty".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> LibraryService {
        LibraryService::in_memory().unwrap()
    }

    fn book() -> NovelDetail {
        NovelDetail {
            source: "test".into(),
            id: "42".into(),
            title: "A book".into(),
            author: Some("Author".into()),
            status: None,
            updated_at: None,
            description: Some("Description".into()),
            cover_url: None,
            tags: Vec::new(),
        }
    }

    #[test]
    fn adds_and_removes_a_bookshelf_entry() {
        let service = service();
        service.add_book(&book()).unwrap();
        assert_eq!(service.list_books().unwrap()[0].book, book());

        service.remove_book("test", "42").unwrap();
        assert!(service.list_books().unwrap().is_empty());
    }

    #[test]
    fn saves_progress_independently_from_bookshelf() {
        let service = service();
        let progress = ReadingProgressInput {
            document_id: "chapter-3".into(),
            document_title: "Chapter 3".into(),
            location: 0.45,
            book_location: 0.245,
        };

        service.save_progress("test", "42", &progress).unwrap();
        let saved = service.get_progress("test", "42").unwrap().unwrap();
        assert_eq!(saved.document_id, "chapter-3");
        assert_eq!(saved.location, 0.45);
        assert_eq!(saved.book_location, 0.245);
    }

    #[test]
    fn rejects_non_finite_progress_before_persistence() {
        let progress = ReadingProgressInput {
            document_id: "chapter-3".into(),
            document_title: "Chapter 3".into(),
            location: f64::NAN,
            book_location: 0.5,
        };

        assert!(matches!(
            service().save_progress("test", "42", &progress),
            Err(LibraryError::InvalidInput(_))
        ));
    }

    #[test]
    fn searches_bookshelf_by_title_only() {
        let service = service();
        service.add_book(&book()).unwrap();

        assert_eq!(service.search_books("a book").unwrap().len(), 1);
        assert!(service.search_books("author").unwrap().is_empty());
        assert!(service.search_books("script").unwrap().is_empty());
        assert!(service.search_books("missing").unwrap().is_empty());
    }

    #[test]
    fn empty_bookshelf_search_returns_all_books() {
        let service = service();
        service.add_book(&book()).unwrap();

        assert_eq!(
            service.search_books("  ").unwrap(),
            service.list_books().unwrap()
        );
    }

    #[test]
    fn rejects_an_excessively_long_bookshelf_query() {
        let query = "x".repeat(101);

        assert!(matches!(
            service().search_books(&query),
            Err(LibraryError::InvalidInput(_))
        ));
    }
}
