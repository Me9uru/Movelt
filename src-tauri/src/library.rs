use std::{path::Path, sync::Mutex};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::novel::NovelDetail;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReadingProgress {
    pub document_id: String,
    pub document_title: String,
    pub location: f64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingProgressInput {
    pub document_id: String,
    pub document_title: String,
    pub location: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BookshelfEntry {
    pub book: NovelDetail,
    pub added_at: String,
    pub progress: Option<ReadingProgress>,
}

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "code", content = "message", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LibraryError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("local library database error: {0}")]
    Database(String),
}

impl From<rusqlite::Error> for LibraryError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error.to_string())
    }
}

pub struct LibraryState {
    connection: Mutex<Connection>,
}

impl LibraryState {
    pub fn new(path: &Path) -> Result<Self, LibraryError> {
        let connection = Connection::open(path)?;
        Self::from_connection(connection)
    }

    fn from_connection(connection: Connection) -> Result<Self, LibraryError> {
        connection.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;

             CREATE TABLE IF NOT EXISTS bookshelf (
               source TEXT NOT NULL,
               book_id TEXT NOT NULL,
               title TEXT NOT NULL,
               author TEXT,
               status TEXT,
               source_updated_at TEXT,
               description TEXT,
               cover_url TEXT,
               added_at TEXT NOT NULL,
               PRIMARY KEY (source, book_id)
             );

             CREATE TABLE IF NOT EXISTS reading_progress (
               source TEXT NOT NULL,
               book_id TEXT NOT NULL,
               document_id TEXT NOT NULL,
               document_title TEXT NOT NULL,
               location REAL NOT NULL CHECK (location >= 0 AND location <= 1),
               updated_at TEXT NOT NULL,
               PRIMARY KEY (source, book_id)
             );",
        )?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>, LibraryError> {
        self.connection
            .lock()
            .map_err(|_| LibraryError::Database("database lock was poisoned".into()))
    }

    fn add_book(&self, book: &NovelDetail) -> Result<(), LibraryError> {
        validate_key(&book.source, &book.id)?;
        self.connection()?.execute(
            "INSERT INTO bookshelf (
               source, book_id, title, author, status, source_updated_at,
               description, cover_url, added_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(source, book_id) DO UPDATE SET
               title = excluded.title,
               author = excluded.author,
               status = excluded.status,
               source_updated_at = excluded.source_updated_at,
               description = excluded.description,
               cover_url = excluded.cover_url",
            params![
                book.source,
                book.id,
                book.title,
                book.author,
                book.status,
                book.updated_at,
                book.description,
                book.cover_url,
            ],
        )?;
        Ok(())
    }

    fn remove_book(&self, source: &str, book_id: &str) -> Result<(), LibraryError> {
        validate_key(source, book_id)?;
        self.connection()?.execute(
            "DELETE FROM bookshelf WHERE source = ?1 AND book_id = ?2",
            params![source, book_id],
        )?;
        Ok(())
    }

    fn list_books(&self) -> Result<Vec<BookshelfEntry>, LibraryError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT b.source, b.book_id, b.title, b.author, b.status,
                    b.source_updated_at, b.description, b.cover_url, b.added_at,
                    p.document_id, p.document_title, p.location, p.updated_at
             FROM bookshelf b
             LEFT JOIN reading_progress p
               ON p.source = b.source AND p.book_id = b.book_id
             ORDER BY COALESCE(p.updated_at, b.added_at) DESC",
        )?;
        let entries = statement
            .query_map([], |row| {
                Ok(BookshelfEntry {
                    book: NovelDetail {
                        source: row.get(0)?,
                        id: row.get(1)?,
                        title: row.get(2)?,
                        author: row.get(3)?,
                        status: row.get(4)?,
                        updated_at: row.get(5)?,
                        description: row.get(6)?,
                        cover_url: row.get(7)?,
                    },
                    added_at: row.get(8)?,
                    progress: match row.get::<_, Option<String>>(9)? {
                        Some(document_id) => Some(ReadingProgress {
                            document_id,
                            document_title: row.get(10)?,
                            location: row.get(11)?,
                            updated_at: row.get(12)?,
                        }),
                        None => None,
                    },
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    fn save_progress(
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
        if !progress.location.is_finite() {
            return Err(LibraryError::InvalidInput(
                "reading location must be a finite number".into(),
            ));
        }
        let location = progress.location.clamp(0.0, 1.0);
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO reading_progress (
               source, book_id, document_id, document_title, location, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(source, book_id) DO UPDATE SET
               document_id = excluded.document_id,
               document_title = excluded.document_title,
               location = excluded.location,
               updated_at = excluded.updated_at",
            params![
                source,
                book_id,
                progress.document_id,
                progress.document_title,
                location,
            ],
        )?;
        drop(connection);
        self.get_progress(source, book_id)?.ok_or_else(|| {
            LibraryError::Database("saved reading progress could not be loaded".into())
        })
    }

    fn get_progress(
        &self,
        source: &str,
        book_id: &str,
    ) -> Result<Option<ReadingProgress>, LibraryError> {
        validate_key(source, book_id)?;
        self.connection()?
            .query_row(
                "SELECT document_id, document_title, location, updated_at
                 FROM reading_progress WHERE source = ?1 AND book_id = ?2",
                params![source, book_id],
                |row| {
                    Ok(ReadingProgress {
                        document_id: row.get(0)?,
                        document_title: row.get(1)?,
                        location: row.get(2)?,
                        updated_at: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
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

#[tauri::command]
pub fn list_bookshelf(state: State<'_, LibraryState>) -> Result<Vec<BookshelfEntry>, LibraryError> {
    state.list_books()
}

#[tauri::command]
pub fn add_to_bookshelf(
    state: State<'_, LibraryState>,
    book: NovelDetail,
) -> Result<(), LibraryError> {
    state.add_book(&book)
}

#[tauri::command]
pub fn remove_from_bookshelf(
    state: State<'_, LibraryState>,
    source: String,
    book_id: String,
) -> Result<(), LibraryError> {
    state.remove_book(&source, &book_id)
}

#[tauri::command]
pub fn get_reading_progress(
    state: State<'_, LibraryState>,
    source: String,
    book_id: String,
) -> Result<Option<ReadingProgress>, LibraryError> {
    state.get_progress(&source, &book_id)
}

#[tauri::command]
pub fn save_reading_progress(
    state: State<'_, LibraryState>,
    source: String,
    book_id: String,
    progress: ReadingProgressInput,
) -> Result<ReadingProgress, LibraryError> {
    state.save_progress(&source, &book_id, &progress)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> LibraryState {
        LibraryState::from_connection(Connection::open_in_memory().unwrap()).unwrap()
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
        }
    }

    #[test]
    fn adds_and_removes_a_bookshelf_entry() {
        let state = state();
        state.add_book(&book()).unwrap();
        assert_eq!(state.list_books().unwrap()[0].book, book());

        state.remove_book("test", "42").unwrap();
        assert!(state.list_books().unwrap().is_empty());
    }

    #[test]
    fn saves_progress_independently_from_bookshelf() {
        let state = state();
        let progress = ReadingProgressInput {
            document_id: "chapter-3".into(),
            document_title: "Chapter 3".into(),
            location: 0.45,
        };

        state.save_progress("test", "42", &progress).unwrap();
        let saved = state.get_progress("test", "42").unwrap().unwrap();
        assert_eq!(saved.document_id, "chapter-3");
        assert_eq!(saved.location, 0.45);
    }
}
