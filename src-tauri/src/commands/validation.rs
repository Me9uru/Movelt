use crate::error::{AppError, Result};

/// 校验并解析作品或章节 ID。
pub(super) fn parse_id(value: &str) -> Result<i64> {
    let id: i64 = value
        .parse()
        .map_err(|_| AppError::invalid_input("作品或章节 ID 必须是正整数"))?;
    if id <= 0 {
        return Err(AppError::invalid_input("作品或章节 ID 必须是正整数"));
    }
    Ok(id)
}

pub(super) fn validate_page_number(value: i64) -> Result<i64> {
    if value > 0 && value < i64::MAX {
        Ok(value)
    } else {
        Err(AppError::invalid_input("页码必须是有效的正整数"))
    }
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
    use super::{parse_id, validate_page_number, validate_page_size};

    #[test]
    fn rejects_non_positive_page_sizes() {
        assert_eq!(validate_page_size(6).expect("positive page size"), 6);
        assert!(validate_page_size(0).is_err());
        assert!(validate_page_size(-1).is_err());
    }

    #[test]
    fn rejects_invalid_ids_and_page_numbers() {
        for id in ["0", "-1", "", "abc", "9223372036854775808"] {
            assert!(parse_id(id).is_err());
        }
        assert_eq!(parse_id("42").unwrap(), 42);
        for page in [0, -1, i64::MIN, i64::MAX] {
            assert!(validate_page_number(page).is_err());
        }
        assert_eq!(validate_page_number(1).unwrap(), 1);
    }
}
