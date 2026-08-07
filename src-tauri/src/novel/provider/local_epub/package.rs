use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use quick_xml::{events::Event, Reader};
use scraper::{Html, Selector};

use crate::novel::error::NovelError;

use super::{
    content::without_chapter_number,
    path::{resolve_inside, resolve_manifest_item, resolve_relative_inside},
};

#[derive(Clone)]
pub(super) struct ManifestItem {
    pub(super) href: String,
    pub(super) media_type: String,
    properties: String,
}

#[derive(Clone, Debug)]
pub(super) struct TocEntry {
    pub(super) title: String,
    pub(super) path: Option<PathBuf>,
    pub(super) children: Vec<TocEntry>,
}

#[derive(Default)]
pub(super) struct Package {
    pub(super) title: Option<String>,
    pub(super) author: Option<String>,
    pub(super) description: Option<String>,
    pub(super) cover_id: Option<String>,
    pub(super) cover_page_href: Option<String>,
    pub(super) base_dir: PathBuf,
    pub(super) manifest: HashMap<String, ManifestItem>,
    pub(super) spine: Vec<String>,
    pub(super) toc: Vec<TocEntry>,
    pub(super) toc_titles: HashMap<PathBuf, String>,
}

pub(super) fn load_package(content: &Path) -> Result<(PathBuf, Package), NovelError> {
    let container = fs::read_to_string(content.join("META-INF/container.xml"))
        .map_err(|_| NovelError::Parse("EPUB 缺少 META-INF/container.xml".into()))?;
    let rootfile = xml_attr(&container, "rootfile", "full-path")
        .ok_or_else(|| NovelError::Parse("EPUB container 未声明 OPF".into()))?;
    let opf_path = resolve_inside(content, &rootfile)?;
    let raw = fs::read_to_string(&opf_path)
        .map_err(|_| NovelError::Parse("无法读取 EPUB OPF 清单".into()))?;
    let mut package = Package {
        base_dir: opf_path
            .parent()
            .and_then(|p| p.strip_prefix(content).ok())
            .map(Path::to_path_buf)
            .ok_or(NovelError::Internal)?,
        ..Package::default()
    };
    let mut xml = Reader::from_str(&raw);
    xml.config_mut().trim_text(true);
    let mut current = String::new();
    loop {
        match xml.read_event() {
            Ok(Event::Start(event)) => {
                current = local_name(event.name().as_ref());
                apply_attrs(&mut package, &current, attrs(&event));
            }
            Ok(Event::Empty(event)) => {
                let name = local_name(event.name().as_ref());
                apply_attrs(&mut package, &name, attrs(&event));
            }
            Ok(Event::Text(text)) => {
                let value = text
                    .unescape()
                    .map_err(|_| NovelError::Parse("EPUB 元数据编码无效".into()))?
                    .trim()
                    .to_owned();
                if !value.is_empty() {
                    match current.as_str() {
                        "title" => package.title = Some(value),
                        "creator" => package.author = Some(value),
                        "description" => package.description = Some(value),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(_)) => current.clear(),
            Ok(Event::Eof) => break,
            Err(_) => return Err(NovelError::Parse("EPUB OPF XML 无法解析".into())),
            _ => {}
        }
    }
    if package.cover_id.is_none() {
        package.cover_id = package
            .manifest
            .iter()
            .find(|(_, item)| {
                item.properties
                    .split_whitespace()
                    .any(|x| x == "cover-image")
            })
            .map(|(id, _)| id.clone());
    }
    package.toc = parse_ncx_toc(content, &package);
    package.toc_titles = parse_nav_titles(content, &package);
    if package.toc_titles.is_empty() {
        collect_toc_titles(&package.toc, &mut package.toc_titles);
    }
    Ok((opf_path, package))
}

fn apply_attrs(package: &mut Package, name: &str, attrs: HashMap<String, String>) {
    match name {
        "item" => {
            if let (Some(id), Some(href)) = (attrs.get("id"), attrs.get("href")) {
                package.manifest.insert(
                    id.clone(),
                    ManifestItem {
                        href: href.clone(),
                        media_type: attrs.get("media-type").cloned().unwrap_or_default(),
                        properties: attrs.get("properties").cloned().unwrap_or_default(),
                    },
                );
            }
        }
        "itemref" => {
            if let Some(id) = attrs.get("idref") {
                package.spine.push(id.clone());
            }
        }
        "meta" if attrs.get("name").map(String::as_str) == Some("cover") => {
            package.cover_id = attrs.get("content").cloned()
        }
        "reference" if attrs.get("type").map(String::as_str) == Some("cover") => {
            package.cover_page_href = attrs.get("href").cloned()
        }
        _ => {}
    }
}

/// EPUB 3 stores its table of contents in an XHTML navigation document rather
/// than the EPUB 2 NCX file. Map its links to the resolved spine file paths.
fn parse_nav_titles(content: &Path, package: &Package) -> HashMap<PathBuf, String> {
    let Some(item) = package.manifest.values().find(|item| {
        item.properties
            .split_whitespace()
            .any(|property| property == "nav")
    }) else {
        return HashMap::new();
    };
    let Ok(nav_path) = resolve_manifest_item(content, package, &item.href) else {
        return HashMap::new();
    };
    let Ok(raw) = fs::read_to_string(&nav_path) else {
        return HashMap::new();
    };
    parse_nav_document(content, &nav_path, &raw)
}

fn parse_nav_document(content: &Path, nav_path: &Path, raw: &str) -> HashMap<PathBuf, String> {
    let document = Html::parse_document(raw);
    let nav_selector = Selector::parse("nav").expect("valid selector");
    let link_selector = Selector::parse("a[href]").expect("valid selector");
    let mut titles = HashMap::new();
    let navs = document.select(&nav_selector).collect::<Vec<_>>();
    let toc_navs = navs
        .iter()
        .copied()
        .filter(|nav| {
            nav.value()
                .attr("epub:type")
                .or_else(|| nav.value().attr("role"))
                .is_some_and(|kind| {
                    kind.split_whitespace()
                        .any(|value| value == "toc" || value == "doc-toc")
                })
        })
        .collect::<Vec<_>>();
    let navs = if toc_navs.is_empty() { navs } else { toc_navs };

    for nav in navs {
        for link in nav.select(&link_selector) {
            let Some(href) = link.value().attr("href") else {
                continue;
            };
            let label = link
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            if label.is_empty() {
                continue;
            }
            if let Ok(path) =
                resolve_relative_inside(content, nav_path.parent().unwrap_or(content), href)
            {
                titles
                    .entry(path)
                    .or_insert_with(|| without_chapter_number(&label));
            }
        }
    }
    titles
}

fn parse_ncx_toc(content: &Path, package: &Package) -> Vec<TocEntry> {
    let Some(item) = package.manifest.values().find(|item| {
        item.media_type.contains("ncx") || item.href.to_ascii_lowercase().ends_with(".ncx")
    }) else {
        return Vec::new();
    };
    let Ok(ncx_path) = resolve_manifest_item(content, package, &item.href) else {
        return Vec::new();
    };
    let Ok(raw) = fs::read_to_string(&ncx_path) else {
        return Vec::new();
    };
    let parent = ncx_path.parent().unwrap_or(content);
    parse_ncx_document(content, parent, &raw)
}

fn parse_ncx_document(content: &Path, parent: &Path, raw: &str) -> Vec<TocEntry> {
    let mut reader = Reader::from_str(raw);
    reader.config_mut().trim_text(true);
    let mut stack = Vec::new();
    let mut in_label = false;
    let mut toc = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => match local_name(event.name().as_ref()).as_str() {
                "navPoint" => stack.push(TocEntry {
                    title: String::new(),
                    path: None,
                    children: Vec::new(),
                }),
                "navLabel" if !stack.is_empty() => in_label = true,
                _ => {}
            },
            Ok(Event::Empty(event)) if local_name(event.name().as_ref()) == "content" => {
                if let (Some(entry), Some(src)) = (stack.last_mut(), attrs(&event).get("src")) {
                    entry.path = resolve_relative_inside(content, parent, src).ok();
                }
            }
            Ok(Event::Text(text)) if in_label => {
                if let (Some(entry), Ok(value)) = (stack.last_mut(), text.unescape()) {
                    entry.title.push_str(value.trim());
                }
            }
            Ok(Event::End(event)) => match local_name(event.name().as_ref()).as_str() {
                "navLabel" => in_label = false,
                "navPoint" => {
                    if let Some(mut entry) = stack.pop() {
                        entry.title = without_chapter_number(&entry.title);
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(entry);
                        } else {
                            toc.push(entry);
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    toc
}

fn collect_toc_titles(entries: &[TocEntry], titles: &mut HashMap<PathBuf, String>) {
    for entry in entries {
        if let (Some(path), false) = (&entry.path, entry.title.is_empty()) {
            titles
                .entry(path.clone())
                .or_insert_with(|| entry.title.clone());
        }
        collect_toc_titles(&entry.children, titles);
    }
}

fn xml_attr(xml: &str, element: &str, key: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event().ok()? {
            Event::Start(e) | Event::Empty(e) if local_name(e.name().as_ref()) == element => {
                return attrs(&e).remove(key)
            }
            Event::Eof => return None,
            _ => {}
        }
    }
}
fn attrs(event: &quick_xml::events::BytesStart<'_>) -> HashMap<String, String> {
    event
        .attributes()
        .flatten()
        .filter_map(|a| {
            Some((
                local_name(a.key.as_ref()),
                a.unescape_value().ok()?.into_owned(),
            ))
        })
        .collect()
}
fn local_name(name: &[u8]) -> String {
    String::from_utf8_lossy(name)
        .rsplit(':')
        .next()
        .unwrap_or_default()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolves_manifest_href_from_opf_directory() {
        let root = Path::new("/tmp/epub-content");
        let package = Package {
            base_dir: PathBuf::from("OEBPS"),
            ..Package::default()
        };
        assert_eq!(
            resolve_manifest_item(root, &package, "text/chapter-1.xhtml").unwrap(),
            root.join("OEBPS/text/chapter-1.xhtml")
        );
    }

    #[test]
    fn extracts_chapter_titles_from_epub3_navigation_document() {
        let root = Path::new("/tmp/epub-content");
        let nav_path = root.join("OEBPS/nav.xhtml");
        let raw = r#"
            <html><body>
              <nav epub:type="toc">
                <ol>
                  <li><a href="text/001.xhtml#start">1. Beginning</a></li>
                  <li><a href="text/002.xhtml">Second chapter</a></li>
                </ol>
              </nav>
              <nav epub:type="landmarks"><a href="cover.xhtml">Cover</a></nav>
            </body></html>
        "#;

        let titles = parse_nav_document(root, &nav_path, raw);

        assert_eq!(
            titles.get(&root.join("OEBPS/text/001.xhtml")),
            Some(&"Beginning".to_owned())
        );
        assert_eq!(
            titles.get(&root.join("OEBPS/text/002.xhtml")),
            Some(&"Second chapter".to_owned())
        );
        assert!(!titles.contains_key(&root.join("OEBPS/cover.xhtml")));
    }

    #[test]
    fn preserves_chapter_and_story_hierarchy_from_ncx() {
        let root = Path::new("/tmp/epub-content");
        let raw = r#"
            <ncx><navMap>
              <navPoint><navLabel><text>Main story</text></navLabel><content src="one.html"/>
                <navPoint><navLabel><text>Chapter one</text></navLabel><content src="one.html#chapter"/>
                  <navPoint><navLabel><text>First story</text></navLabel><content src="two.html"/></navPoint>
                  <navPoint><navLabel><text>Second story</text></navLabel><content src="three.html"/></navPoint>
                </navPoint>
              </navPoint>
            </navMap></ncx>
        "#;

        let toc = parse_ncx_document(root, root, raw);

        assert_eq!(toc[0].title, "Main story");
        assert_eq!(toc[0].children[0].title, "Chapter one");
        assert_eq!(toc[0].children[0].children[0].title, "First story");
        assert_eq!(
            toc[0].children[0].children[1].path,
            Some(root.join("three.html"))
        );
    }
}
