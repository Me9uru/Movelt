use crate::error::{AppError, Result};

/// 校验并解析作品或章节 ID。
pub(super) fn parse_id(value: &str) -> Result<i64> {
    value
        .parse()
        .map_err(|_| AppError::invalid_input("作品 ID 必须是数字"))
}

/// 校验分页大小，避免将空值或负数透传给官方接口。
pub(super) fn validate_page_size(value: i64) -> Result<i64> {
    if value > 0 {
        Ok(value)
    } else {
        Err(AppError::invalid_input("分页大小必须大于 0"))
    }
}

#[cfg(test)]
mod tests {
    use super::validate_page_size;

    #[test]
    fn rejects_non_positive_page_sizes() {
        assert_eq!(validate_page_size(6).expect("positive page size"), 6);
        assert!(validate_page_size(0).is_err());
    }
}
