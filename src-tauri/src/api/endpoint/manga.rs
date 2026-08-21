use serde_json::{json, Value};

use crate::error::Result;

use super::super::connection::OfficialClient;

impl OfficialClient {
    pub(crate) async fn manga_list(
        &self,
        query: Option<String>,
        page: i64,
        browse_type: &str,
    ) -> Result<Value> {
        let search = matches!(browse_type, "SEARCH" | "TAGS");
        if search {
            return self
                .hub(
                    "SearchComicSeries",
                    json!({
                        "KeyWords": query.unwrap_or_default(), "Page": page, "Size": 30,
                        "Mode": if browse_type == "TAGS" { "tags" } else { "fuzzy" },
                    }),
                )
                .await;
        }
        let order = match browse_type {
            "POPULAR" => "view",
            "NEW" => "new",
            _ => "latest",
        };
        self.hub(
            "GetComicList",
            json!({ "Page": page, "Size": 30, "Order": order }),
        )
        .await
    }

    pub(crate) async fn manga_info(&self, id: i64) -> Result<Value> {
        self.hub("GetComicInfo", json!({ "Id": id })).await
    }

    pub(crate) async fn manga_content(&self, chapter_id: i64, skip: i64) -> Result<Value> {
        self.hub(
            "GetComicContent",
            json!({ "Cid": chapter_id, "Skip": skip, "Take": 12 }),
        )
        .await
    }

    pub(crate) async fn save_manga_position(
        &self,
        manga_id: i64,
        chapter_id: i64,
        page: i64,
    ) -> Result<()> {
        self.hub(
            "SaveReadPosition",
            json!({ "Bid": manga_id, "Cid": chapter_id, "XPath": page.to_string() }),
        )
        .await
        .map(|_| ())
    }
}
