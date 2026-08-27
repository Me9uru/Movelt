use serde_json::Value;

use crate::{
    dto::user::{Growth, User},
    error::Result,
    mapping::value::{number, optional_number, optional_string, string},
};

/// 将官方用户数据映射为应用 DTO。
pub(crate) fn user(value: Value) -> Result<User> {
    Ok(User {
        id: number(&value, "Id"),
        user_name: string(&value, "UserName"),
        avatar: optional_string(&value, "Avatar"),
        email: optional_string(&value, "Email"),
        invite_code: optional_string(&value, "InviteCode"),
        user_group: value
            .get("Role")
            .and_then(|role| optional_string(role, "Name")),
        register_at: optional_string(&value, "RegisterAt"),
        growth: growth(value.get("Growth")),
    })
}

fn growth(value: Option<&Value>) -> Option<Growth> {
    let value = value?;
    value.as_object()?;
    Some(Growth {
        exp: number(value, "Exp"),
        coin: number(value, "Coin"),
        level: number(value, "Level"),
        growth_level: number(value, "GrowthLevel"),
        current_level_exp: number(value, "CurrentLevelExp"),
        next_level_exp: optional_number(value, "NextLevelExp"),
        sign_streak: number(value, "SignStreak"),
        today_signed: value
            .get("TodaySigned")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::user;

    #[test]
    fn maps_full_user_profile() {
        let mapped = user(json!({
            "Id": 42,
            "UserName": "读者",
            "Email": "reader@example.com",
            "InviteCode": "welcome",
            "Role": { "Name": "普通用户" },
            "RegisterAt": "2025-01-02T03:04:05Z",
            "Growth": {
                "Exp": 10,
                "Coin": 20,
                "Level": 2,
                "GrowthLevel": 1,
                "CurrentLevelExp": 0,
                "NextLevelExp": 100,
                "SignStreak": 3,
                "TodaySigned": true,
            },
        }))
        .expect("profile should map");

        assert_eq!(mapped.user_group.as_deref(), Some("普通用户"));
        assert_eq!(mapped.register_at.as_deref(), Some("2025-01-02T03:04:05Z"));
        assert_eq!(mapped.growth.expect("growth").coin, 20);
    }
}
