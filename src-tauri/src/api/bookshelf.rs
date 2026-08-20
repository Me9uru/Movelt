use serde_json::{json, Value};

use crate::error::Result;

use super::OfficialClient;

impl OfficialClient {
    pub(crate) async fn bookshelf(&self) -> Result<Value> {
        self.hub("GetBookShelf", json!({})).await
    }

    pub(crate) async fn save_bookshelf(&self, items: Vec<Value>, version: &str) -> Result<()> {
        self.hub("SaveBookShelf", json!({ "data": items, "ver": version }))
            .await
            .map(|_| ())
    }
}
