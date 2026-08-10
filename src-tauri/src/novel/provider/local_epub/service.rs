use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use sha2::{Digest, Sha256};

use crate::novel::{
    domain::{ChapterContent, ChapterSummary, NovelDetail, NovelOverview, Volume},
    error::NovelError,
};

use super::{
    archive, content,
    package::{load_package, Package, TocEntry},
    path::{resolve_inside, resolve_manifest_item},
};

const MAX_ASSET_BYTES: u64 = 16 * 1024 * 1024;

/// EPUBs are copied into the application data directory, so their original location is never used again.
pub(crate) struct LocalEpubSource {
    root: PathBuf,
}

impl LocalEpubSource {
    pub(crate) const SOURCE_ID: &'static str = "local_epub";

    pub(crate) fn new(root: PathBuf) -> Result<Self, NovelError> {
        fs::create_dir_all(&root).map_err(io_error)?;
        Ok(Self { root })
    }

    pub(crate) fn import(&self, source_path: &str) -> Result<NovelOverview, NovelError> {
        let path = Path::new(source_path);
        if path
            .extension()
            .and_then(|x| x.to_str())
            .map(|x| x.eq_ignore_ascii_case("epub"))
            != Some(true)
        {
            return Err(NovelError::invalid_input("请选择 .epub 文件"));
        }
        let mut input =
            File::open(path).map_err(|_| NovelError::invalid_input("无法读取所选 EPUB 文件"))?;
        let mut hasher = Sha256::new();
        let mut data = Vec::new();
        input.read_to_end(&mut data).map_err(io_error)?;
        hasher.update(&data);
        let id = format!("{:x}", hasher.finalize());
        let book_dir = self.root.join(&id);
        if book_dir.join("content").is_dir() {
            return self.overview(&id);
        }
        let staging = self.root.join(format!(".{id}.importing"));
        let _ = fs::remove_dir_all(&staging);
        fs::create_dir_all(staging.join("content")).map_err(io_error)?;
        let result = (|| {
            fs::write(staging.join("book.epub"), &data).map_err(io_error)?;
            archive::extract(Cursor::new(&data), &staging.join("content"))?;
            self.overview_at(
                &id,
                &staging.join("content"),
                path.file_stem().and_then(|x| x.to_str()),
            )?;
            fs::rename(&staging, &book_dir).map_err(io_error)?;
            self.overview(&id)
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(&staging);
        }
        result
    }

    pub(crate) fn overview(&self, book_id: &str) -> Result<NovelOverview, NovelError> {
        self.overview_at(book_id, &self.content_dir(book_id)?, None)
    }

    pub(crate) fn chapter(
        &self,
        book_id: &str,
        chapter_id: &str,
    ) -> Result<ChapterContent, NovelError> {
        let content = self.content_dir(book_id)?;
        let (_, package) = load_package(&content)?;
        let item = package
            .manifest
            .get(chapter_id)
            .ok_or(NovelError::NotFound)?;
        if !package.spine.iter().any(|id| id == chapter_id) {
            return Err(NovelError::NotFound);
        }
        let chapter_path = resolve_manifest_item(&content, &package, &item.href)?;
        let raw = fs::read_to_string(&chapter_path).map_err(|_| NovelError::NotFound)?;
        let (parsed_title, nodes) = content::parse_chapter(&content, &chapter_path, &raw)?;
        // The navigation document is the canonical chapter title.  The XHTML
        // `<title>` is often a generic filename such as "chapter x-x".
        let title = package
            .toc_titles
            .get(&chapter_path)
            .cloned()
            .filter(|title| !title.is_empty())
            .or_else(|| (!parsed_title.is_empty()).then_some(parsed_title))
            .unwrap_or_else(|| chapter_id.to_owned());
        Ok(ChapterContent {
            source: Self::SOURCE_ID.into(),
            novel_id: book_id.into(),
            chapter_id: chapter_id.into(),
            title,
            nodes,
        })
    }

    pub(crate) fn remove(&self, book_id: &str) -> Result<(), NovelError> {
        let path = self
            .content_dir(book_id)?
            .parent()
            .ok_or(NovelError::NotFound)?
            .to_path_buf();
        fs::remove_dir_all(path).map_err(io_error)
    }

    pub(crate) fn asset_data_url(
        &self,
        book_id: &str,
        resource_path: &str,
    ) -> Result<String, NovelError> {
        let content = self.content_dir(book_id)?;
        let candidate = Path::new(resource_path);
        let relative = candidate
            .strip_prefix(&content)
            .map_err(|_| NovelError::NotFound)?;
        let path = resolve_inside(&content, relative)?;
        let metadata = fs::metadata(&path).map_err(|_| NovelError::NotFound)?;
        if metadata.len() > MAX_ASSET_BYTES {
            return Err(NovelError::Parse("EPUB 图片超过 16 MB 限制".into()));
        }
        let bytes = fs::read(&path).map_err(|_| NovelError::NotFound)?;
        Ok(format!(
            "data:{};base64,{}",
            asset_mime_type(&path),
            BASE64.encode(bytes)
        ))
    }

    fn content_dir(&self, book_id: &str) -> Result<PathBuf, NovelError> {
        if book_id.len() != 64 || !book_id.bytes().all(|x| x.is_ascii_hexdigit()) {
            return Err(NovelError::invalid_input("无效的本地书籍 ID"));
        }
        let content = self.root.join(book_id).join("content");
        if content.is_dir() {
            Ok(content)
        } else {
            Err(NovelError::NotFound)
        }
    }

    fn overview_at(
        &self,
        book_id: &str,
        content: &Path,
        fallback_title: Option<&str>,
    ) -> Result<NovelOverview, NovelError> {
        let (_, package) = load_package(content)?;
        let title = package
            .title
            .clone()
            .unwrap_or_else(|| fallback_title.unwrap_or("未命名 EPUB").to_owned());
        let cover_url = package
            .cover_id
            .as_ref()
            .and_then(|id| package.manifest.get(id).cloned())
            .and_then(|item| resolve_manifest_item(content, &package, &item.href).ok())
            .or_else(|| {
                package.cover_page_href.as_ref().and_then(|href| {
                    resolve_manifest_item(content, &package, href)
                        .ok()
                        .and_then(|page| content::cover_image_from_page(content, &page))
                })
            })
            .map(|path| path.to_string_lossy().into_owned());
        let chapters = package
            .spine
            .iter()
            .filter_map(|id| package.manifest.get(id).map(|item| (id, item)))
            .filter(|(_, item)| {
                item.media_type.contains("html")
                    || item.href.ends_with(".xhtml")
                    || item.href.ends_with(".html")
            })
            .map(|(id, item)| {
                let path = resolve_manifest_item(content, &package, &item.href).ok();
                let title = path
                    .as_ref()
                    .and_then(|path| package.toc_titles.get(path).cloned())
                    .or_else(|| {
                        path.as_ref()
                            .and_then(|path| fs::read_to_string(path).ok())
                            .map(|raw| content::chapter_title(&raw))
                            .filter(|x| !x.is_empty())
                    })
                    .unwrap_or_else(|| id.clone());
                ChapterSummary {
                    id: id.clone(),
                    title,
                }
            })
            .collect::<Vec<_>>();
        if chapters.is_empty() {
            return Err(NovelError::Parse("EPUB 缺少可阅读的 spine 正文".into()));
        }
        let volumes = volumes_from_toc(content, &package).unwrap_or_else(|| {
            vec![Volume {
                title: "正文".into(),
                chapters,
                sections: Vec::new(),
            }]
        });
        Ok(NovelOverview {
            detail: NovelDetail {
                source: Self::SOURCE_ID.into(),
                id: book_id.into(),
                title,
                author: package.author,
                status: None,
                updated_at: None,
                description: package.description,
                cover_url,
                tags: Vec::new(),
            },
            volumes,
        })
    }
}

fn volumes_from_toc(content: &Path, package: &Package) -> Option<Vec<Volume>> {
    let spine_by_path = package
        .spine
        .iter()
        .filter_map(|id| {
            let item = package.manifest.get(id)?;
            let path = resolve_manifest_item(content, package, &item.href).ok()?;
            Some((path, id.clone()))
        })
        .collect::<HashMap<_, _>>();
    let volumes = package
        .toc
        .iter()
        .filter(|entry| !entry.children.is_empty())
        .filter_map(|entry| volume_from_toc_entry(entry, &spine_by_path))
        .collect::<Vec<_>>();
    (!volumes.is_empty()).then_some(volumes)
}

fn volume_from_toc_entry(
    entry: &TocEntry,
    spine_by_path: &HashMap<PathBuf, String>,
) -> Option<Volume> {
    let mut seen = HashSet::new();
    let chapters = entry
        .children
        .iter()
        .filter(|child| child.children.is_empty())
        .filter_map(|child| {
            let id = spine_by_path.get(child.path.as_ref()?)?.clone();
            seen.insert(id.clone()).then_some(ChapterSummary {
                id,
                title: child.title.clone(),
            })
        })
        .collect::<Vec<_>>();
    let sections = entry
        .children
        .iter()
        .filter(|child| !child.children.is_empty())
        .filter_map(|child| volume_from_toc_entry(child, spine_by_path))
        .collect::<Vec<_>>();
    (!chapters.is_empty() || !sections.is_empty()).then_some(Volume {
        title: entry.title.clone(),
        chapters,
        sections,
    })
}

fn asset_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => "image/png",
    }
}

fn io_error(error: std::io::Error) -> NovelError {
    NovelError::Parse(format!("无法保存或读取 EPUB 数据：{error}"))
}
