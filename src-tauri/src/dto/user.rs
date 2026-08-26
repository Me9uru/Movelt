use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct User {
    pub id: i64,
    pub user_name: String,
    pub avatar: Option<String>,
    pub email: Option<String>,
    pub invite_code: Option<String>,
    pub user_group: Option<String>,
    pub register_at: Option<String>,
    pub growth: Option<Growth>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Growth {
    pub exp: i64,
    pub coin: i64,
    pub level: i64,
    pub growth_level: i64,
    pub current_level_exp: i64,
    pub next_level_exp: Option<i64>,
    pub sign_streak: i64,
    pub today_signed: bool,
}
