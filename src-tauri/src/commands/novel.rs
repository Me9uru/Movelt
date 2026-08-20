use serde_json::json;
use tauri::State;
use url::Url;

use crate::{
    api::OfficialClient,
    dto::novel::{
        ChapterSummary, DiscoveryList, NovelOverview, NovelSummary, ReaderDocument, Volume,
    },
    error::{AppError, Result},
};

use super::common::{array, novel, number, optional_string, page, parse_id, position, string};

#[tauri::command]
pub(crate) async fn get_latest(
    client: State<'_, OfficialClient>,
    page_number: Option<i64>,
) -> Result<DiscoveryList> {
    page(
        client
            .hub(
                "GetLatestBookList",
                json!({"Page": page_number.unwrap_or(1), "Size": 24}),
            )
            .await?,
    )
}
#[tauri::command]
pub(crate) async fn get_ranking(
    client: State<'_, OfficialClient>,
    sort: String,
    page_number: Option<i64>,
) -> Result<DiscoveryList> {
    page(
        client
            .hub(
                "GetBookList",
                json!({"Page": page_number.unwrap_or(1), "Size": 24, "Order": sort}),
            )
            .await?,
    )
}
#[tauri::command]
pub(crate) async fn get_rank(
    client: State<'_, OfficialClient>,
    days: i64,
) -> Result<Vec<NovelSummary>> {
    Ok(client
        .hub("GetRank", json!({"Days": days}))
        .await?
        .as_array()
        .map(|items| items.iter().map(novel).collect())
        .unwrap_or_default())
}
#[tauri::command]
pub(crate) async fn search_novels(
    client: State<'_, OfficialClient>,
    query: String,
    page_number: Option<i64>,
    tags: bool,
) -> Result<DiscoveryList> {
    let method = if tags {
        "GetBookListByTags"
    } else {
        "GetBookList"
    };
    page(
        client
            .hub(
                method,
                json!({"Page": page_number.unwrap_or(1), "Size": 24, "KeyWords": query}),
            )
            .await?,
    )
}
#[tauri::command]
pub(crate) async fn get_reader_overview(
    client: State<'_, OfficialClient>,
    book_id: String,
) -> Result<NovelOverview> {
    let response = client
        .hub("GetBookInfo", json!({"Id": parse_id(&book_id)?}))
        .await?;
    let book = response
        .get("Book")
        .ok_or_else(|| AppError::InvalidResponse("书籍详情缺失".into()))?;
    let mut read_position = position(response.get("ReadPosition"));
    let chapters = array(book, "Chapter");
    if let Some(position) = &mut read_position {
        if let Some(index) = chapters
            .iter()
            .position(|chapter| number(chapter, "Id").to_string() == position.chapter_id)
        {
            position.chapter_id = (index + 1).to_string();
        }
    }
    Ok(NovelOverview {
        detail: novel(book),
        volumes: vec![Volume {
            title: "章节".into(),
            chapters: chapters
                .iter()
                .enumerate()
                .map(|(index, chapter)| ChapterSummary {
                    id: (index + 1).to_string(),
                    title: string(chapter, "Title"),
                })
                .collect(),
            sections: vec![],
        }],
        read_position,
    })
}
#[tauri::command]
pub(crate) async fn get_reader_document(
    client: State<'_, OfficialClient>,
    book_id: String,
    document_id: String,
) -> Result<ReaderDocument> {
    let response = client
        .hub(
            "GetNovelContent",
            json!({"Bid": parse_id(&book_id)?, "SortNum": parse_id(&document_id)?}),
        )
        .await?;
    let chapter = response
        .get("Chapter")
        .ok_or_else(|| AppError::InvalidResponse("章节内容缺失".into()))?;
    Ok(ReaderDocument {
        id: format!("{book_id}:{document_id}"),
        book_id,
        chapter_id: document_id,
        server_chapter_id: number(chapter, "Id").to_string(),
        title: string(chapter, "Title"),
        html: ammonia::clean(&string(chapter, "Content")),
        font_url: chapter_font_url(chapter),
        read_position: position(response.get("ReadPosition")),
    })
}

fn chapter_font_url(chapter: &serde_json::Value) -> Option<String> {
    let font = optional_string(chapter, "Font")?;
    if font.starts_with('/') {
        return Some(format!("https://api.lightnovel.life{font}"));
    }

    let url = Url::parse(&font).ok()?;
    let trusted_host = matches!(
        url.host_str(),
        Some("api.lightnovel.life" | "cf-api.lightnovel.life" | "img.lightnovel.life")
    );
    (url.scheme() == "https" && trusted_host).then_some(font)
}
#[tauri::command]
pub(crate) async fn save_read_position(
    client: State<'_, OfficialClient>,
    book_id: String,
    chapter_id: String,
    xpath: String,
) -> Result<()> {
    client
        .hub(
            "SaveReadPosition",
            json!({"Bid": parse_id(&book_id)?, "Cid": parse_id(&chapter_id)?, "XPath": xpath}),
        )
        .await
        .map(|_| ())
}
