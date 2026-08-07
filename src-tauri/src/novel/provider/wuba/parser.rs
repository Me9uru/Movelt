use super::SOURCE_ID;
use crate::novel::domain::{
    ChapterContent, ChapterNode, ChapterSummary, NovelDetail, NovelOverview, NovelSummary,
    SearchResult, Volume,
};
use crate::novel::error::NovelError;
use scraper::{ElementRef, Html, Selector};
use url::Url;

pub(super) fn search(html: &str, base_url: &Url) -> Result<SearchResult, NovelError> {
    let document = Html::parse_document(html);
    let item_selector = selector("dl.B")?;
    let title_selector = selector("dt a")?;
    let image_selector = selector("dd.imgB img")?;
    let items = document
        .select(&item_selector)
        .filter_map(|item| parse_search_item(item, &title_selector, &image_selector, base_url))
        .collect();

    Ok(SearchResult {
        page: 1,
        total_pages: 1,
        items,
    })
}

pub(super) fn overview(
    html: &str,
    novel_id: &str,
    base_url: &Url,
) -> Result<NovelOverview, NovelError> {
    let document = Html::parse_document(html);
    let title = meta_content(&document, "og:title")
        .or(optional_text(&document, "dl.B dt.FCol")?)
        .ok_or_else(|| NovelError::Parse("missing novel title".into()))?;
    let author = meta_content(&document, "og:novel:author");
    let status = meta_content(&document, "og:novel:status");
    let updated_at = meta_content(&document, "og:novel:update_time");
    let description = optional_text(&document, ".nrjj .e")?;
    let cover_url = document
        .select(&selector("dl.B dd.imgB img")?)
        .next()
        .and_then(|element| image_url(element, base_url));
    let chapters = parse_chapters(&document, novel_id, base_url)?;

    Ok(NovelOverview {
        detail: NovelDetail {
            source: SOURCE_ID.into(),
            id: novel_id.into(),
            title,
            author,
            status,
            updated_at,
            description,
            cover_url,
        },
        volumes: vec![Volume {
            title: "正文".into(),
            chapters,
            sections: Vec::new(),
        }],
    })
}

pub(super) fn chapter_page_count(html: &str, chapter_id: &str) -> Result<u32, NovelError> {
    let document = Html::parse_document(html);
    let link_selector = selector("#PageSet a")?;
    let page_count = document
        .select(&link_selector)
        .filter_map(|link| page_from_href(link.value().attr("href")?, chapter_id))
        .max()
        .unwrap_or(1);
    Ok(page_count)
}

pub(super) fn chapter(
    pages: &[String],
    novel_id: &str,
    chapter_id: &str,
) -> Result<ChapterContent, NovelError> {
    let first = pages
        .first()
        .ok_or_else(|| NovelError::Parse("chapter has no pages".into()))?;
    let first_document = Html::parse_document(first);
    let title = optional_text(&first_document, "h1.sh1")?
        .ok_or_else(|| NovelError::Parse("missing chapter title".into()))?;
    let paragraph_selector = selector(".TxtContent p")?;
    let mut paragraphs: Vec<String> = Vec::new();

    for html in pages {
        let document = Html::parse_document(html);
        let mut page_paragraphs = document
            .select(&paragraph_selector)
            .map(element_text)
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>();
        if paragraphs.last().is_some_and(|last| !ends_sentence(last)) && !page_paragraphs.is_empty()
        {
            let continuation = page_paragraphs.remove(0);
            if let Some(last) = paragraphs.last_mut() {
                last.push_str(&continuation);
            }
        }
        paragraphs.extend(page_paragraphs);
    }

    if paragraphs.is_empty() {
        return Err(NovelError::Parse("chapter content is empty".into()));
    }
    Ok(ChapterContent {
        source: SOURCE_ID.into(),
        novel_id: novel_id.into(),
        chapter_id: chapter_id.into(),
        title,
        nodes: paragraphs
            .into_iter()
            .map(|text| ChapterNode::Paragraph { text })
            .collect(),
    })
}

fn parse_search_item(
    item: ElementRef<'_>,
    title_selector: &Selector,
    image_selector: &Selector,
    base_url: &Url,
) -> Option<NovelSummary> {
    let link = item.select(title_selector).next()?;
    let id = novel_id_from_href(link.value().attr("href")?, base_url)?;
    let title = element_text(link);
    (!title.is_empty()).then_some(NovelSummary {
        source: SOURCE_ID.into(),
        id,
        title,
        cover_url: item
            .select(image_selector)
            .next()
            .and_then(|element| image_url(element, base_url)),
    })
}

fn parse_chapters(
    document: &Html,
    novel_id: &str,
    base_url: &Url,
) -> Result<Vec<ChapterSummary>, NovelError> {
    let chapter_selector = selector("ul.MLlist li a")?;
    let chapters = document
        .select(&chapter_selector)
        .filter_map(|link| {
            let id = chapter_id_from_href(link.value().attr("href")?, novel_id, base_url)?;
            let title = element_text(link);
            (!title.is_empty()).then_some(ChapterSummary { id, title })
        })
        .collect::<Vec<_>>();
    if chapters.is_empty() {
        Err(NovelError::Parse("catalogue is empty".into()))
    } else {
        Ok(chapters)
    }
}

fn meta_content(document: &Html, property: &str) -> Option<String> {
    let selector = Selector::parse(&format!("meta[property=\"{property}\"]")).ok()?;
    document
        .select(&selector)
        .next()?
        .value()
        .attr("content")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(Into::into)
}

fn novel_id_from_href(href: &str, base_url: &Url) -> Option<String> {
    let path = source_path(href, base_url)?;
    let mut segments = path.trim_matches('/').split('/');
    let (prefix, id) = (segments.next()?, segments.next()?);
    (prefix == "wuba" && segments.next().is_none() && is_ascii_digits(id)).then(|| id.into())
}

fn chapter_id_from_href(href: &str, novel_id: &str, base_url: &Url) -> Option<String> {
    let path = source_path(href, base_url)?;
    let mut segments = path.trim_matches('/').split('/');
    let (prefix, book, filename) = (segments.next()?, segments.next()?, segments.next()?);
    let id = filename.strip_suffix(".html")?;
    (prefix == "wubashu" && book == novel_id && segments.next().is_none() && is_ascii_digits(id))
        .then(|| id.into())
}

fn page_from_href(href: &str, chapter_id: &str) -> Option<u32> {
    let filename = href.rsplit('/').next()?;
    let stem = filename.strip_suffix(".html")?;
    if stem == chapter_id {
        return Some(1);
    }
    stem.strip_prefix(chapter_id)?
        .strip_prefix('-')?
        .parse()
        .ok()
}

fn source_path(value: &str, base_url: &Url) -> Option<String> {
    base_url
        .join(value)
        .ok()
        .filter(|url| url.host_str() == base_url.host_str())
        .map(|url| url.path().into())
}

fn image_url(element: ElementRef<'_>, base_url: &Url) -> Option<String> {
    let value = element
        .value()
        .attr("src")
        .filter(|value| !value.is_empty())
        .or_else(|| element.value().attr("data-src"))?;
    let url = base_url.join(value).ok()?;
    (matches!(url.scheme(), "http" | "https") && url.host_str() == base_url.host_str())
        .then(|| url.into())
}

fn optional_text(document: &Html, css: &str) -> Result<Option<String>, NovelError> {
    Ok(document
        .select(&selector(css)?)
        .next()
        .map(element_text)
        .filter(|value| !value.is_empty()))
}

fn selector(css: &str) -> Result<Selector, NovelError> {
    Selector::parse(css).map_err(|_| NovelError::Internal)
}

fn element_text(element: ElementRef<'_>) -> String {
    element
        .text()
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

fn ends_sentence(value: &str) -> bool {
    value.ends_with(['。', '！', '？', '…', '”', '’', '：', ':'])
}

fn is_ascii_digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_url() -> Url {
        Url::parse("http://m.5859ycdh.com/").unwrap()
    }

    #[test]
    fn parses_search_result_and_lazy_cover() {
        let html = r#"<dl class="B">
            <dd class="imgB"><img data-src="/files/50919s.jpg"></dd>
            <dt><a href="/wuba/50919/">蛊真人</a></dt>
        </dl>"#;
        let result = search(html, &base_url()).unwrap();
        assert_eq!(result.items[0].id, "50919");
        assert_eq!(result.items[0].title, "蛊真人");
        assert_eq!(
            result.items[0].cover_url.as_deref(),
            Some("http://m.5859ycdh.com/files/50919s.jpg")
        );
    }

    #[test]
    fn parses_detail_and_catalogue_from_one_page() {
        let html = r#"
            <meta property="og:title" content="蛊真人">
            <meta property="og:novel:author" content="蛊真人">
            <meta property="og:novel:status" content="连载中">
            <meta property="og:novel:update_time" content="2019-05-30">
            <dl class="B"><dd class="imgB"><img src="/cover.jpg"></dd></dl>
            <div class="nrjj"><p class="e">作品简介</p></div>
            <ul class="MLlist">
              <li><a href="/wubashu/50919/37262160.html">第一节</a></li>
            </ul>
        "#;
        let result = overview(html, "50919", &base_url()).unwrap();
        assert_eq!(result.detail.author.as_deref(), Some("蛊真人"));
        assert_eq!(result.volumes[0].chapters[0].id, "37262160");
    }

    #[test]
    fn detects_and_merges_chapter_pages() {
        let first = r#"
            <h1 class="sh1">第一节</h1>
            <div class="TxtContent"><p>第一段。</p><p>准</p>广告</div>
            <div id="PageSet"><a href="37262160-2.html">下一页</a></div>
        "#;
        let second = r#"
            <div class="TxtContent"><p>备。</p><p>第二段。</p>温馨提示</div>
            <div id="PageSet"><a href="37262160-1.html">上一页</a></div>
        "#;
        assert_eq!(chapter_page_count(first, "37262160").unwrap(), 2);
        let result = chapter(&[first.into(), second.into()], "50919", "37262160").unwrap();
        assert_eq!(
            result.nodes,
            vec![
                ChapterNode::Paragraph {
                    text: "第一段。".into()
                },
                ChapterNode::Paragraph {
                    text: "准备。".into()
                },
                ChapterNode::Paragraph {
                    text: "第二段。".into()
                }
            ]
        );
    }
}
