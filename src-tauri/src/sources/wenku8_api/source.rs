use serde::Deserialize;

use super::client::Wenku8Client;
use crate::discovery::domain::{DiscoveryList, HealthStatus, PageInfo, RecommendBlock};
use crate::domain::{
    ChapterContent, ChapterNode, ChapterSummary, NovelDetail, NovelOverview, NovelSummary, Volume,
};
use crate::error::NovelError;

pub(crate) const SOURCE_ID: &str = "wenku8_api";
const LANG: &str = "zh_CN";
pub(super) const SORTS: &[&str] = &[
    "allvisit",
    "allvote",
    "monthvisit",
    "monthvote",
    "weekvisit",
    "weekvote",
    "dayvisit",
    "dayvote",
    "postdate",
    "lastupdate",
    "goodnum",
    "size",
    "fullflag",
    "anime",
];

pub(crate) struct Wenku8ApiSource {
    client: Wenku8Client,
}

#[derive(Deserialize)]
struct ApiHealth {
    logged_in: bool,
}

struct FlexibleId(u64);

struct IdVisitor;

#[derive(Deserialize)]
struct ApiSearchResult {
    results: Vec<ApiSearchItem>,
    page_control: ApiPageControl,
}

#[derive(Deserialize)]
struct ApiSearchItem {
    aid: FlexibleId,
    title: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    last_updated: Option<String>,
    #[serde(default)]
    status: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    intro_preview: String,
}

#[derive(Deserialize)]
struct ApiPageControl {
    now: u32,
    previous: u32,
    next: u32,
    begin: u32,
    end: u32,
}

#[derive(Deserialize)]
struct ApiRecommendBlock {
    title: String,
    list: Vec<ApiNovelCover>,
}

#[derive(Deserialize)]
struct ApiNovelCover {
    title: String,
    aid: FlexibleId,
}

#[derive(Deserialize)]
struct ApiNovelInfo {
    title: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    last_updated: Option<String>,
    #[serde(default)]
    intro: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Deserialize)]
struct ApiNovelIndex {
    volumes: Vec<ApiVolume>,
}

#[derive(Deserialize)]
struct ApiVolume {
    title: String,
    chapters: Vec<ApiChapter>,
}

#[derive(Deserialize)]
struct ApiChapter {
    cid: FlexibleId,
    title: String,
}

impl<'de> serde::de::Visitor<'de> for IdVisitor {
    type Value = FlexibleId;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a positive numeric id or numeric string")
    }

    fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
        (value > 0)
            .then_some(FlexibleId(value))
            .ok_or_else(|| E::custom("id must be positive"))
    }

    fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
        u64::try_from(value)
            .map_err(E::custom)
            .and_then(|value| self.visit_u64(value))
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        value
            .parse::<u64>()
            .map_err(E::custom)
            .and_then(|value| self.visit_u64(value))
    }
}

impl Wenku8ApiSource {
    pub(crate) fn new(base_url: &str) -> Result<Self, NovelError> {
        Ok(Self {
            client: Wenku8Client::new(base_url)?,
        })
    }

    pub(crate) async fn health(&self) -> Result<HealthStatus, NovelError> {
        let response: ApiHealth = self.client.json("health", &[]).await?;
        Ok(HealthStatus {
            logged_in: response.logged_in,
            base_url: self.client.base_url(),
        })
    }

    pub(crate) async fn recommend(&self) -> Result<Vec<RecommendBlock>, NovelError> {
        let blocks: Vec<ApiRecommendBlock> = self
            .client
            .json("recommend", &[("lang", LANG.into())])
            .await?;
        Ok(blocks
            .into_iter()
            .map(|block| RecommendBlock {
                title: block.title,
                items: block
                    .list
                    .into_iter()
                    .map(|item| self.cover_summary(item))
                    .collect(),
            })
            .collect())
    }

    pub(crate) async fn ranking(&self, sort: &str, page: u32) -> Result<DiscoveryList, NovelError> {
        validate_sort(sort)?;
        self.list_impl("novel/list", &[("sort", sort.into())], page)
            .await
    }

    pub(crate) async fn category(
        &self,
        tag: &str,
        sort: &str,
        page: u32,
    ) -> Result<DiscoveryList, NovelError> {
        validate_text(tag, "tag", 40)?;
        validate_sort(sort)?;
        self.list_impl(
            "category",
            &[("tag", tag.into()), ("sort", sort.into())],
            page,
        )
        .await
    }

    pub(crate) async fn search_mode(
        &self,
        query: &str,
        method: &str,
        page: u32,
    ) -> Result<DiscoveryList, NovelError> {
        validate_text(query, "query", 100)?;
        if !matches!(method, "articlename" | "author") {
            return Err(NovelError::invalid_input(
                "search method must be articlename or author",
            ));
        }
        self.list_impl(
            "search",
            &[("keyword", query.into()), ("method", method.into())],
            page,
        )
        .await
    }

    fn summary(&self, item: ApiSearchItem) -> NovelSummary {
        let id = item.aid.to_string();
        NovelSummary {
            source: SOURCE_ID.into(),
            cover_url: Some(self.cover_url(&id)),
            id,
            title: item.title,
            author: nonempty(item.author),
            status: nonempty(item.status),
            updated_at: item.last_updated,
            description: nonempty(item.intro_preview),
            tags: item.tags,
        }
    }

    fn cover_summary(&self, item: ApiNovelCover) -> NovelSummary {
        let id = item.aid.to_string();
        NovelSummary {
            source: SOURCE_ID.into(),
            cover_url: Some(self.cover_url(&id)),
            id,
            title: item.title,
            author: None,
            status: None,
            updated_at: None,
            description: None,
            tags: Vec::new(),
        }
    }

    fn cover_url(&self, aid: &str) -> String {
        self.client
            .url(&format!("novel/cover/{aid}"), &[])
            .expect("validated base URL and numeric aid")
            .into()
    }

    pub(crate) async fn cover_data_url(&self, aid: &str) -> Result<String, NovelError> {
        validate_id(aid, "novel")?;
        self.client
            .image_data_url(&format!("novel/cover/{aid}"), &[])
            .await
    }

    fn picture_url(&self, remote_url: &str) -> Result<String, NovelError> {
        Ok(self
            .client
            .url("picture", &[("url", remote_url.into())])?
            .into())
    }
}

impl Wenku8ApiSource {
    pub(crate) async fn overview(&self, novel_id: &str) -> Result<NovelOverview, NovelError> {
        validate_id(novel_id, "novel")?;
        let query = [("lang", LANG.into())];
        let info_path = format!("novel/info/{novel_id}");
        let index_path = format!("novel/index/{novel_id}");
        let (info, index): (ApiNovelInfo, ApiNovelIndex) = futures::try_join!(
            self.client.json(&info_path, &query),
            self.client.json(&index_path, &query),
        )?;
        let detail = NovelDetail {
            source: SOURCE_ID.into(),
            id: novel_id.into(),
            title: info.title,
            author: nonempty(info.author),
            status: nonempty(info.status),
            updated_at: info.last_updated,
            description: nonempty(info.intro),
            cover_url: Some(self.cover_url(novel_id)),
            tags: info.tags,
        };
        let volumes = index
            .volumes
            .into_iter()
            .map(|volume| Volume {
                title: volume.title,
                chapters: volume
                    .chapters
                    .into_iter()
                    .map(|chapter| ChapterSummary {
                        id: chapter.cid.to_string(),
                        title: chapter.title,
                    })
                    .collect(),
                sections: Vec::new(),
            })
            .collect();
        Ok(NovelOverview { detail, volumes })
    }

    pub(crate) async fn chapter_with_title(
        &self,
        novel_id: &str,
        chapter_id: &str,
        title: Option<&str>,
    ) -> Result<ChapterContent, NovelError> {
        validate_id(novel_id, "novel")?;
        validate_id(chapter_id, "chapter")?;
        let query = [("lang", LANG.into())];
        let primary = self
            .client
            .text(&format!("novel/content/{novel_id}/{chapter_id}"), &query)
            .await;
        let text = match primary {
            Ok(text) => text,
            Err(NovelError::NotLoggedIn) => return Err(NovelError::NotLoggedIn),
            Err(_) => {
                self.client
                    .text(
                        &format!("novel/content_via_full/{novel_id}/{chapter_id}"),
                        &query,
                    )
                    .await?
            }
        };
        Ok(ChapterContent {
            source: SOURCE_ID.into(),
            novel_id: novel_id.into(),
            chapter_id: chapter_id.into(),
            title: title
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("章节")
                .into(),
            nodes: parse_content(&text, |url| self.picture_url(url))?,
        })
    }
}

// Kept separate so request validation and response mapping are easy to test.
impl Wenku8ApiSource {
    async fn list_impl(
        &self,
        path: &str,
        extra: &[(&str, String)],
        page: u32,
    ) -> Result<DiscoveryList, NovelError> {
        validate_page(page)?;
        let mut query = extra.to_vec();
        query.push(("page", page.to_string()));
        query.push(("lang", LANG.into()));
        let response: ApiSearchResult = self.client.json(path, &query).await?;
        Ok(DiscoveryList {
            items: response
                .results
                .into_iter()
                .map(|item| self.summary(item))
                .collect(),
            pagination: response.page_control.into(),
        })
    }
}

impl<'de> Deserialize<'de> for FlexibleId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(IdVisitor)
    }
}

impl std::fmt::Display for FlexibleId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl From<ApiPageControl> for PageInfo {
    fn from(value: ApiPageControl) -> Self {
        Self {
            page: value.now,
            previous: (value.previous > 0).then_some(value.previous),
            next: (value.next > 0).then_some(value.next),
            first: value.begin.max(1),
            last: value.end.max(1),
        }
    }
}
fn validate_id(id: &str, kind: &str) -> Result<(), NovelError> {
    if !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit()) {
        Ok(())
    } else {
        Err(NovelError::invalid_input(format!(
            "{kind} id must contain digits only"
        )))
    }
}
fn validate_page(page: u32) -> Result<(), NovelError> {
    if (1..=1_000).contains(&page) {
        Ok(())
    } else {
        Err(NovelError::invalid_input("page must be between 1 and 1000"))
    }
}
fn validate_sort(sort: &str) -> Result<(), NovelError> {
    if SORTS.contains(&sort) {
        Ok(())
    } else {
        Err(NovelError::invalid_input("unsupported ranking sort"))
    }
}
fn validate_text(value: &str, name: &str, max: usize) -> Result<(), NovelError> {
    let length = value.trim().chars().count();
    if (1..=max).contains(&length) {
        Ok(())
    } else {
        Err(NovelError::invalid_input(format!(
            "{name} must contain between 1 and {max} characters"
        )))
    }
}
fn nonempty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

fn parse_content<F>(text: &str, proxy: F) -> Result<Vec<ChapterNode>, NovelError>
where
    F: Fn(&str) -> Result<String, NovelError>,
{
    const MARKER: &str = "<!--image-->";
    let mut nodes = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(MARKER) {
        push_paragraphs(&rest[..start], &mut nodes);
        let after = &rest[start + MARKER.len()..];
        if let Some(end) = after.find(MARKER) {
            let url = after[..end].trim();
            if !url.is_empty() {
                nodes.push(ChapterNode::Image {
                    url: proxy(url)?,
                    alt: None,
                });
            }
            rest = &after[end + MARKER.len()..];
        } else {
            push_paragraphs(&rest[start..], &mut nodes);
            rest = "";
        }
    }
    push_paragraphs(rest, &mut nodes);
    Ok(nodes)
}
fn push_paragraphs(text: &str, nodes: &mut Vec<ChapterNode>) {
    nodes.extend(
        text.split('\n')
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|text| ChapterNode::Paragraph { text: text.into() }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    #[test]
    fn accepts_numeric_and_string_ids() {
        let numeric: ApiNovelCover = serde_json::from_str(r#"{"title":"a","aid":42}"#).unwrap();
        let string: ApiNovelCover = serde_json::from_str(r#"{"title":"a","aid":"42"}"#).unwrap();
        assert_eq!(numeric.aid.to_string(), "42");
        assert_eq!(string.aid.to_string(), "42");
    }
    #[test]
    fn keeps_paragraph_and_image_order() {
        let nodes = parse_content(
            "one\n<!--image-->https://wenku8.com/a.jpg<!--image-->\ntwo",
            |url| Ok(format!("proxy:{url}")),
        )
        .unwrap();
        assert!(
            matches!(&nodes[..], [ChapterNode::Paragraph { text }, ChapterNode::Image { .. }, ChapterNode::Paragraph { text: last }] if text == "one" && last == "two")
        );
    }
    #[test]
    fn rejects_invalid_inputs() {
        assert!(validate_id("../1", "novel").is_err());
        assert!(validate_page(0).is_err());
        assert!(validate_sort("unknown").is_err());
        assert!(validate_text(" ", "query", 100).is_err());
    }

    #[test]
    fn falls_back_to_full_content_after_primary_failure() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            for index in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0_u8; 2048];
                let size = stream.read(&mut request).unwrap();
                let request = String::from_utf8_lossy(&request[..size]);
                if index == 0 {
                    assert!(request.contains("/novel/content/42/7?lang=zh_CN"));
                    write!(stream, "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                } else {
                    assert!(request.contains("/novel/content_via_full/42/7?lang=zh_CN"));
                    let body = "fallback paragraph";
                    write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .unwrap();
                }
            }
        });
        let source = Wenku8ApiSource::new(&format!("http://{address}")).unwrap();
        let chapter = tauri::async_runtime::block_on(source.chapter_with_title(
            "42",
            "7",
            Some("Known title"),
        ))
        .unwrap();
        assert_eq!(chapter.title, "Known title");
        assert!(
            matches!(&chapter.nodes[..], [ChapterNode::Paragraph { text }] if text == "fallback paragraph")
        );
        server.join().unwrap();
    }
}
