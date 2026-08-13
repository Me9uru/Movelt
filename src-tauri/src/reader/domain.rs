use crate::domain::{ChapterContent, ChapterNode};
use serde::{Deserialize, Serialize};

/// Source-independent payload consumed by the reader frontend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReaderDocument {
    pub id: String,
    pub source_id: String,
    pub book_id: String,
    pub title: String,
    pub blocks: Vec<ChapterNode>,
}

impl From<ChapterContent> for ReaderDocument {
    fn from(chapter: ChapterContent) -> Self {
        Self {
            id: format!(
                "{}:{}:{}",
                chapter.source, chapter.novel_id, chapter.chapter_id
            ),
            source_id: chapter.source,
            book_id: chapter.novel_id,
            title: chapter.title,
            blocks: chapter.nodes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_chapter_content_for_the_reader() {
        let document = ReaderDocument::from(ChapterContent {
            source: "local_epub".into(),
            novel_id: "book".into(),
            chapter_id: "chapter".into(),
            title: "Chapter".into(),
            nodes: Vec::new(),
        });

        assert_eq!(document.id, "local_epub:book:chapter");
        assert_eq!(document.source_id, "local_epub");
        assert_eq!(document.book_id, "book");
    }
}
