use serde_json::{json, Value};

use crate::{
    dto::common::{Order, SearchMode},
    error::Result,
};

use super::super::connection::OfficialClient;

impl OfficialClient {
    pub(crate) async fn list_novels(&self, page: i64, size: i64, order: Order) -> Result<Value> {
        self.hub(
            "GetBookList",
            json!({ "Page": page, "Size": size, "Order": order.hub_order() }),
        )
        .await
    }

    pub(crate) async fn get_novel_rank(&self, days: i64) -> Result<Value> {
        self.hub("GetRank", json!({ "Days": days })).await
    }

    pub(crate) async fn search_novels(
        &self,
        query: String,
        page: i64,
        size: i64,
        mode: SearchMode,
    ) -> Result<Value> {
        let method = match mode {
            SearchMode::Fuzzy | SearchMode::Exact => "GetBookList",
            SearchMode::Title => "GetBookListByTitle",
            SearchMode::Author => "GetBookListByAuthor",
            SearchMode::Name => "GetBookListByName",
            SearchMode::Tags => "GetBookListByTags",
        };
        let keywords = if matches!(mode, SearchMode::Exact) {
            format!("\"{query}\"")
        } else {
            query
        };
        self.hub(
            method,
            json!({ "Page": page, "Size": size, "KeyWords": keywords }),
        )
        .await
    }

    pub(crate) async fn get_novel_info(&self, id: i64) -> Result<Value> {
        self.hub("GetBookInfo", json!({ "Id": id })).await
    }

    pub(crate) async fn get_novel_content(
        &self,
        book_id: i64,
        chapter: i64,
        convert: Option<&str>,
    ) -> Result<Value> {
        self.hub(
            "GetNovelContent",
            json!({ "Bid": book_id, "SortNum": chapter, "Convert": convert }),
        )
        .await
    }

    pub(crate) async fn save_novel_position(
        &self,
        book_id: i64,
        chapter_id: i64,
        xpath: String,
    ) -> Result<()> {
        super::save_read_position(self, book_id, chapter_id, xpath).await
    }

    pub(crate) async fn get_books_by_ids(&self, ids: &[i64], kind: Option<&str>) -> Result<Value> {
        let mut payload = json!({ "Ids": ids });
        if let Some(kind) = kind {
            payload["Type"] = Value::String(kind.into());
        }
        self.hub("GetBookListByIds", payload).await
    }
}
