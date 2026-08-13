use serde::{Deserialize, Serialize};

use crate::domain::NovelSummary;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PageInfo {
    pub page: u32,
    pub previous: Option<u32>,
    pub next: Option<u32>,
    pub first: u32,
    pub last: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct DiscoveryList {
    pub items: Vec<NovelSummary>,
    pub pagination: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RecommendBlock {
    pub title: String,
    pub items: Vec<NovelSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HealthStatus {
    pub logged_in: bool,
    pub base_url: String,
}
