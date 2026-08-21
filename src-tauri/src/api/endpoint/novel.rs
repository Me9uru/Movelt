use serde_json::{json, Value};

use crate::error::Result;

use super::super::connection::OfficialClient;

impl OfficialClient {
    pub(crate) async fn latest_novels(&self, page: i64) -> Result<Value> {
        self.hub("GetLatestBookList", json!({ "Page": page, "Size": 24 }))
            .await
    }

    pub(crate) async fn ranked_novels(&self, order: String, page: i64) -> Result<Value> {
        self.hub(
            "GetBookList",
            json!({ "Page": page, "Size": 24, "Order": order }),
        )
        .await
    }

    pub(crate) async fn novel_rank(&self, days: i64) -> Result<Value> {
        self.hub("GetRank", json!({ "Days": days })).await
    }

    pub(crate) async fn search_novels(
        &self,
        query: String,
        page: i64,
        tags: bool,
    ) -> Result<Value> {
        let method = if tags {
            "GetBookListByTags"
        } else {
            "GetBookList"
        };
        self.hub(
            method,
            json!({ "Page": page, "Size": 24, "KeyWords": query }),
        )
        .await
    }

    pub(crate) async fn novel_info(&self, id: i64) -> Result<Value> {
        self.hub("GetBookInfo", json!({ "Id": id })).await
    }

    pub(crate) async fn novel_content(
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
        self.hub(
            "SaveReadPosition",
            json!({ "Bid": book_id, "Cid": chapter_id, "XPath": xpath }),
        )
        .await
        .map(|_| ())
    }

    pub(crate) async fn books_by_ids(&self, ids: &[i64], kind: Option<&str>) -> Result<Value> {
        let mut payload = json!({ "Ids": ids });
        if let Some(kind) = kind {
            payload["Type"] = Value::String(kind.into());
        }
        self.hub("GetBookListByIds", payload).await
    }
}
