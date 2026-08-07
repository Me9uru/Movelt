use std::{path::Path, sync::Mutex};

use rusqlite::{params, Connection, OptionalExtension};

use super::{
    domain::{BookshelfEntry, ReadingProgress, ReadingProgressInput},
    error::LibraryError,
};
use crate::novel::domain::NovelDetail;

pub(super) struct SqliteLibraryRepository {
    connection: Mutex<Connection>,
}

impl SqliteLibraryRepository {
    pub(super) fn open(path: &Path) -> Result<Self, LibraryError> {
        Self::from_connection(Connection::open(path)?)
    }

    #[cfg(test)]
    pub(super) fn in_memory() -> Result<Self, LibraryError> {
        Self::from_connection(Connection::open_in_memory()?)
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
               book_location REAL NOT NULL DEFAULT 0 CHECK (book_location >= 0 AND book_location <= 1),
               updated_at TEXT NOT NULL,
               PRIMARY KEY (source, book_id)
             );",
        )?;
        let has_book_location = {
            let mut statement = connection.prepare("PRAGMA table_info(reading_progress)")?;
            let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
            columns
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .any(|column| column == "book_location")
        };
        if !has_book_location {
            connection.execute(
                "ALTER TABLE reading_progress ADD COLUMN book_location REAL NOT NULL DEFAULT 0
                 CHECK (book_location >= 0 AND book_location <= 1)",
                [],
            )?;
        }
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>, LibraryError> {
        self.connection
            .lock()
            .map_err(|_| LibraryError::Database("database lock was poisoned".into()))
    }

    pub(super) fn add_book(&self, book: &NovelDetail) -> Result<(), LibraryError> {
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

    pub(super) fn remove_book(&self, source: &str, book_id: &str) -> Result<(), LibraryError> {
        let connection = self.connection()?;
        connection.execute(
            "DELETE FROM bookshelf WHERE source = ?1 AND book_id = ?2",
            params![source, book_id],
        )?;
        connection.execute(
            "DELETE FROM reading_progress WHERE source = ?1 AND book_id = ?2",
            params![source, book_id],
        )?;
        Ok(())
    }

    pub(super) fn list_books(&self) -> Result<Vec<BookshelfEntry>, LibraryError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT b.source, b.book_id, b.title, b.author, b.status,
                    b.source_updated_at, b.description, b.cover_url, b.added_at,
                    p.document_id, p.document_title, p.location, p.book_location, p.updated_at
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
                            book_location: row.get(12)?,
                            updated_at: row.get(13)?,
                        }),
                        None => None,
                    },
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    pub(super) fn save_progress(
        &self,
        source: &str,
        book_id: &str,
        progress: &ReadingProgressInput,
        location: f64,
        book_location: f64,
    ) -> Result<ReadingProgress, LibraryError> {
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO reading_progress (
               source, book_id, document_id, document_title, location, book_location, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT(source, book_id) DO UPDATE SET
               document_id = excluded.document_id,
               document_title = excluded.document_title,
               location = excluded.location,
               book_location = excluded.book_location,
               updated_at = excluded.updated_at",
            params![
                source,
                book_id,
                progress.document_id,
                progress.document_title,
                location,
                book_location,
            ],
        )?;
        drop(connection);
        self.get_progress(source, book_id)?.ok_or_else(|| {
            LibraryError::Database("saved reading progress could not be loaded".into())
        })
    }

    pub(super) fn get_progress(
        &self,
        source: &str,
        book_id: &str,
    ) -> Result<Option<ReadingProgress>, LibraryError> {
        self.connection()?
            .query_row(
                "SELECT document_id, document_title, location, book_location, updated_at
                 FROM reading_progress WHERE source = ?1 AND book_id = ?2",
                params![source, book_id],
                |row| {
                    Ok(ReadingProgress {
                        document_id: row.get(0)?,
                        document_title: row.get(1)?,
                        location: row.get(2)?,
                        book_location: row.get(3)?,
                        updated_at: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }
}

impl From<rusqlite::Error> for LibraryError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_book_location_to_an_existing_progress_table() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE reading_progress (
                   source TEXT NOT NULL,
                   book_id TEXT NOT NULL,
                   document_id TEXT NOT NULL,
                   document_title TEXT NOT NULL,
                   location REAL NOT NULL CHECK (location >= 0 AND location <= 1),
                   updated_at TEXT NOT NULL,
                   PRIMARY KEY (source, book_id)
                 );
                 INSERT INTO reading_progress VALUES (
                   'test', '42', 'chapter-3', 'Chapter 3', 0.45, '2026-01-01T00:00:00Z'
                 );",
            )
            .unwrap();

        let repository = SqliteLibraryRepository::from_connection(connection).unwrap();
        let progress = repository.get_progress("test", "42").unwrap().unwrap();

        assert_eq!(progress.location, 0.45);
        assert_eq!(progress.book_location, 0.0);
    }
}
