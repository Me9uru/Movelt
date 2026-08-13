use std::path::{Component, Path, PathBuf};

use crate::error::NovelError;

use super::package::Package;

pub(super) fn resolve_inside(base: &Path, path: impl AsRef<Path>) -> Result<PathBuf, NovelError> {
    let mut output = base.to_path_buf();
    for component in path.as_ref().components() {
        match component {
            Component::Normal(value) => output.push(value),
            Component::CurDir => {}
            _ => return Err(NovelError::Parse("EPUB 包含不安全的资源路径".into())),
        }
    }
    Ok(output)
}

pub(super) fn resolve_relative_inside(
    root: &Path,
    parent: &Path,
    raw: &str,
) -> Result<PathBuf, NovelError> {
    if raw.contains("://") || raw.starts_with("data:") || raw.starts_with('/') {
        return Err(NovelError::Parse("EPUB 不支持外部资源".into()));
    }
    let raw = raw
        .split('#')
        .next()
        .unwrap_or_default()
        .split('?')
        .next()
        .unwrap_or_default();
    let mut relative = parent
        .strip_prefix(root)
        .map_err(|_| NovelError::Internal)?
        .to_path_buf();
    for component in Path::new(raw).components() {
        match component {
            Component::Normal(value) => relative.push(value),
            Component::CurDir => {}
            Component::ParentDir => {
                if !relative.pop() {
                    return Err(NovelError::Parse("EPUB 包含不安全的资源路径".into()));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(NovelError::Parse("EPUB 包含不安全的资源路径".into()));
            }
        }
    }
    resolve_inside(root, relative)
}

pub(super) fn resolve_manifest_item(
    root: &Path,
    package: &Package,
    href: &str,
) -> Result<PathBuf, NovelError> {
    resolve_relative_inside(root, &root.join(&package.base_dir), href)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_relative_parent_without_leaving_epub_root() {
        let root = Path::new("/tmp/epub-content");
        assert_eq!(
            resolve_relative_inside(root, &root.join("OEBPS/text"), "../images/cover.jpg").unwrap(),
            root.join("OEBPS/images/cover.jpg"),
        );
    }

    #[test]
    fn rejects_relative_path_that_escapes_epub_root() {
        let root = Path::new("/tmp/epub-content");
        assert!(resolve_relative_inside(root, root, "../../outside.jpg").is_err());
    }
}
