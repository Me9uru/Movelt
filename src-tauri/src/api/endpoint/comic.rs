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
        self.hub("GetComicInfo", json!({ "Id": id })).await
    }

    /// 获取漫画系列及其分卷；系列标题不是数字书籍 ID。
    pub(crate) async fn get_comic_series_info(&self, series_title: &str) -> Result<Value> {
        self.hub(
            "GetComicSeriesInfo",
            json!({ "SeriesTitle": series_title, "Order": Order::Latest.hub_order() }),
        )
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
