mod bookshelf;
mod comic;
mod novel;
mod user;

use super::connection::OfficialClient;
use crate::error::Result;
use serde_json::json;

/// 保存官方统一的阅读位置；各领域负责将自身位置表示转换为 XPath。
async fn save_read_position(
    client: &OfficialClient,
    book_id: i64,
    chapter_id: i64,
    xpath: String,
) -> Result<()> {
    client
        .hub(
            "SaveReadPosition",
            json!({ "Bid": book_id, "Cid": chapter_id, "XPath": xpath }),
        )
        .await
        .map(|_| ())
}
