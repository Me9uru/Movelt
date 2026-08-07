use std::{
    fs,
    path::{Path, PathBuf},
};

use scraper::{Html, Selector};

use crate::novel::{domain::ChapterNode, error::NovelError};

use super::path::resolve_relative_inside;

pub(super) fn parse_chapter(
    root: &Path,
    chapter_path: &Path,
    raw: &str,
) -> Result<(String, Vec<ChapterNode>), NovelError> {
    let document = Html::parse_document(raw);
    let selector =
        Selector::parse("body p, body h1, body h2, body h3, body h4, body h5, body h6, body img")
            .expect("valid selector");
    let parent = chapter_path.parent().ok_or(NovelError::NotFound)?;
    let mut nodes = Vec::new();
    for element in document.select(&selector) {
        if element.value().name() == "img" {
            let src = element.value().attr("src").unwrap_or("");
            if let Ok(image) = resolve_relative_inside(root, parent, src) {
                nodes.push(ChapterNode::Image {
                    url: image.to_string_lossy().into_owned(),
                    alt: element.value().attr("alt").map(str::to_owned),
                });
            }
        } else {
            let text = text_of(element);
            if !text.is_empty() {
                nodes.push(ChapterNode::Paragraph { text });
            }
        }
    }
    if nodes.is_empty() {
        let body = Selector::parse("body").expect("valid selector");
        if let Some(body) = document.select(&body).next() {
            let text = text_of(body);
            if !text.is_empty() {
                nodes.push(ChapterNode::Paragraph { text });
            }
        }
    }
    if nodes.is_empty() {
        return Err(NovelError::Parse("章节没有可阅读的正文".into()));
    }
    Ok((chapter_title(raw), nodes))
}

pub(super) fn cover_image_from_page(content: &Path, page: &Path) -> Option<PathBuf> {
    let raw = fs::read_to_string(page).ok()?;
    let document = Html::parse_document(&raw);
    let image = document.select(&Selector::parse("img").ok()?).next()?;
    resolve_relative_inside(content, page.parent()?, image.value().attr("src")?).ok()
}

pub(super) fn chapter_title(raw: &str) -> String {
    let document = Html::parse_document(raw);
    let selector = Selector::parse("body h1, body h2, body h3, body h4, body h5, body h6, title")
        .expect("valid selector");
    document
        .select(&selector)
        .next()
        .map(text_of)
        .map(|title| without_chapter_number(&title))
        .unwrap_or_default()
}

pub(super) fn without_chapter_number(title: &str) -> String {
    let trimmed = title.trim();
    let digits = trimmed.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return trimmed.to_owned();
    }
    let remainder = trimmed[digits..]
        .trim_start_matches(|character: char| {
            character.is_whitespace() || matches!(character, '.' | '、' | ':' | '：' | '-' | '—')
        })
        .trim();
    if remainder.is_empty() {
        trimmed.to_owned()
    } else {
        remainder.to_owned()
    }
}

fn text_of(element: scraper::ElementRef<'_>) -> String {
    element
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn removes_leading_chapter_number_from_toc_title() {
        assert_eq!(without_chapter_number("12   龙与淑女"), "龙与淑女");
        assert_eq!(without_chapter_number("第十二章"), "第十二章");
    }
}
