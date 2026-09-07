use serde_json::{json, Value};

use crate::{
    dto::common::{Order, SearchMode},
    error::Result,
};

use super::super::connection::OfficialClient;

impl OfficialClient {
    pub(crate) async fn list_comics(&self, page: i64, size: i64, order: Order) -> Result<Value> {
        self.hub(
            "GetComicList",
            json!({ "Page": page, "Size": size, "Order": order.hub_order() }),
        )
        .await
    }

    pub(crate) async fn search_comics(
        &self,
        query: String,
        page: i64,
        size: i64,
        mode: SearchMode,
    ) -> Result<Value> {
        self.hub(
            "SearchComicSeries",
            json!({ "KeyWords": query, "Page": page, "Size": size, "Mode": mode.hub_mode() }),
        )
        .await
    }

    pub(crate) async fn get_comic_info(&self, id: i64) -> Result<Value> {
        self.hub("GetBookInfo", json!({ "Id": id })).await
    }

    /// 通过系列名定位代表分卷，再由统一详情中的 Series 获取完整系列。
    pub(crate) async fn find_comic_series_book(&self, series_title: &str) -> Result<Value> {
        self.hub("GetBooksBySeries", comic_series_payload(series_title))
            .await
    }

    pub(crate) async fn get_comic_content(&self, chapter_id: i64, skip: i64) -> Result<Value> {
        self.hub(
            "GetComicContent",
            json!({ "Cid": chapter_id, "Skip": skip, "Take": 12 }),
        )
        .await
    }

    pub(crate) async fn save_comic_position(
        &self,
        comic_id: i64,
        chapter_id: i64,
        page: i64,
    ) -> Result<()> {
        super::save_read_position(self, comic_id, chapter_id, page.to_string()).await
    }
}

fn comic_series_payload(series_title: &str) -> Value {
    json!({ "SeriesName": series_title, "Type": "Comic", "Page": 1, "Size": 1, "Order": "latest" })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locates_series_with_explicit_comic_type() {
        assert_eq!(
            comic_series_payload("系列"),
            json!({
                "SeriesName": "系列", "Type": "Comic", "Page": 1, "Size": 1, "Order": "latest"
            })
        );
    }
}
