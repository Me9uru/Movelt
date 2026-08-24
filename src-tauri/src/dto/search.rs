use serde::Deserialize;

/// 受支持的作品搜索字段；命令输入使用 camelCase，官方接口映射在端点层处理。
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum BookSearchMode {
    Title,
    Author,
    Tags,
}

impl BookSearchMode {
    pub(crate) fn novel_hub_method(self) -> &'static str {
        match self {
            Self::Title => "GetBookListByName",
            Self::Author => "GetBookListByAuthor",
            Self::Tags => "GetBookListByTags",
        }
    }

    pub(crate) fn manga_hub_mode(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Author => "author",
            Self::Tags => "tags",
        }
    }
}
