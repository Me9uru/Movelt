use super::client::{ApiChapter, ApiNovel, ApiVolume};
use super::SOURCE_ID;
use crate::novel::{
    domain::{
        ChapterContent, ChapterNode, ChapterSummary, NovelDetail, NovelSummary, SearchResult,
        Volume,
    },
    error::NovelError,
};
use ego_tree::NodeRef;
use scraper::{Html, Node};

const PAGE_SIZE: usize = 20;
const BROKEN_IMAGE_HOST: &str = "lnvoel.animes.garden";
const IMAGE_HOST: &str = "lnovel.animes.garden";

pub fn search(novels: &[ApiNovel], query: &str, page: u32) -> SearchResult {
    let query = query.to_lowercase();
    let matches = novels
        .iter()
        .filter(|novel| {
            novel.name.to_lowercase().contains(&query)
                || novel
                    .authors
                    .iter()
                    .any(|author| author.name.to_lowercase().contains(&query))
        })
        .collect::<Vec<_>>();
    let total_pages = matches.len().div_ceil(PAGE_SIZE).max(1) as u32;
    let start = (page.saturating_sub(1) as usize).saturating_mul(PAGE_SIZE);
    let items = matches
        .into_iter()
        .skip(start)
        .take(PAGE_SIZE)
        .map(|novel| NovelSummary {
            source: SOURCE_ID.into(),
            id: novel.nid.to_string(),
            title: novel.name.clone(),
            cover_url: novel.cover.clone(),
        })
        .collect();

    SearchResult {
        page,
        total_pages,
        items,
    }
}

pub fn detail(novel: &ApiNovel) -> NovelDetail {
    let author = novel
        .authors
        .iter()
        .filter(|author| author.position == "author")
        .map(|author| author.name.as_str())
        .collect::<Vec<_>>()
        .join("、");
    let status = novel
        .labels
        .iter()
        .find(|label| matches!(label.as_str(), "连载" | "完结" | "完本"))
        .cloned();

    NovelDetail {
        source: SOURCE_ID.into(),
        id: novel.nid.to_string(),
        title: novel.name.clone(),
        author: (!author.is_empty()).then_some(author),
        status,
        updated_at: novel.updated_at.clone(),
        description: novel.description.as_deref().map(html_to_text),
        cover_url: novel.cover.clone(),
    }
}

pub fn volume(volume: &ApiVolume) -> Volume {
    Volume {
        title: volume.name.clone(),
        chapters: volume
            .chapters
            .iter()
            .map(|chapter| ChapterSummary {
                id: chapter.cid.to_string(),
                title: chapter.title.clone(),
            })
            .collect(),
    }
}

pub fn chapter(chapter: &ApiChapter) -> Result<ChapterContent, NovelError> {
    let fragment = Html::parse_fragment(&chapter.content);
    let mut nodes = Vec::new();
    let mut paragraph = String::new();
    for child in fragment.tree.root().children() {
        walk_content(child, &mut paragraph, &mut nodes);
    }
    flush_paragraph(&mut paragraph, &mut nodes);
    if nodes.is_empty() {
        return Err(NovelError::Parse("chapter content is empty".into()));
    }

    Ok(ChapterContent {
        source: SOURCE_ID.into(),
        novel_id: chapter.nid.to_string(),
        chapter_id: chapter.cid.to_string(),
        title: chapter.title.clone(),
        nodes,
    })
}

fn html_to_text(html: &str) -> String {
    let fragment = Html::parse_fragment(html);
    normalize(fragment.root_element().text())
}

fn walk_content(node: NodeRef<'_, Node>, paragraph: &mut String, nodes: &mut Vec<ChapterNode>) {
    match node.value() {
        Node::Text(text) => append_text(paragraph, text),
        Node::Element(element) => {
            let name = element.name();
            if matches!(name, "script" | "style") {
                return;
            }
            if name == "img" {
                flush_paragraph(paragraph, nodes);
                if let Some(src) = element.attr("src") {
                    nodes.push(ChapterNode::Image {
                        url: normalize_image_url(src),
                        alt: element.attr("alt").map(str::to_owned),
                    });
                }
                return;
            }
            if name == "br" {
                flush_paragraph(paragraph, nodes);
                return;
            }
            for child in node.children() {
                walk_content(child, paragraph, nodes);
            }
            if matches!(name, "p" | "div" | "section" | "article" | "center") {
                flush_paragraph(paragraph, nodes);
            }
        }
        _ => {
            for child in node.children() {
                walk_content(child, paragraph, nodes);
            }
        }
    }
}

fn normalize_image_url(src: &str) -> String {
    src.replace(BROKEN_IMAGE_HOST, IMAGE_HOST)
}

fn append_text(paragraph: &mut String, text: &str) {
    let clean = normalize(std::iter::once(text));
    if clean.is_empty() {
        return;
    }
    if !paragraph.is_empty() {
        paragraph.push(' ');
    }
    paragraph.push_str(&clean);
}

fn flush_paragraph(paragraph: &mut String, nodes: &mut Vec<ChapterNode>) {
    let text = paragraph.trim().to_owned();
    paragraph.clear();
    if !text.is_empty() {
        nodes.push(ChapterNode::Paragraph { text });
    }
}

fn normalize<'a>(parts: impl Iterator<Item = &'a str>) -> String {
    parts
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_preserves_paragraph_and_image_order() {
        let result = chapter(&ApiChapter {
            nid: 4649,
            cid: 274118,
            title: "第一话".into(),
            content: "<p>第一段</p><img src=\"https://example.com/a.jpg\"><p>第二段</p>".into(),
        })
        .unwrap();
        assert_eq!(result.nodes.len(), 3);
        assert!(matches!(result.nodes[1], ChapterNode::Image { .. }));
    }

    #[test]
    fn chapter_repairs_known_bilinovel_image_host_typo() {
        let result = chapter(&ApiChapter {
            nid: 2139,
            cid: 129550,
            title: "插图".into(),
            content: "<img src=\"https://lnvoel.animes.garden/bili/img3/2/2139/image.jpg\">".into(),
        })
        .unwrap();

        assert_eq!(
            result.nodes,
            vec![ChapterNode::Image {
                url: "https://lnovel.animes.garden/bili/img3/2/2139/image.jpg".into(),
                alt: None,
            }]
        );
    }
}
