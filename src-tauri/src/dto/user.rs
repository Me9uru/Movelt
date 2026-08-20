use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct User {
    pub id: i64,
    pub user_name: String,
    pub avatar: Option<String>,
    pub email: Option<String>,
}
