use std::{
    fs::{self, File},
    io::{Read, Seek},
    path::Path,
};

use zip::ZipArchive;

use crate::novel::error::NovelError;

use super::path::resolve_inside;

const MAX_ENTRIES: usize = 10_000;
const MAX_UNCOMPRESSED_BYTES: u64 = 256 * 1024 * 1024;

pub(super) fn extract<R: Read + Seek>(reader: R, output: &Path) -> Result<(), NovelError> {
    let mut archive = ZipArchive::new(reader)
        .map_err(|_| NovelError::Parse("所选文件不是有效的 EPUB ZIP 归档".into()))?;
    if archive.len() > MAX_ENTRIES {
        return Err(NovelError::Parse("EPUB 包含过多文件".into()));
    }
    let mut total = 0_u64;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|_| NovelError::Parse("EPUB 归档无法读取".into()))?;
        if entry.is_dir() {
            continue;
        }
        total = total
            .checked_add(entry.size())
            .ok_or_else(|| NovelError::Parse("EPUB 文件过大".into()))?;
        if total > MAX_UNCOMPRESSED_BYTES {
            return Err(NovelError::Parse("EPUB 解压后超过 256 MB 限制".into()));
        }
        let name = entry
            .enclosed_name()
            .ok_or_else(|| NovelError::Parse("EPUB 包含不安全的文件路径".into()))?
            .to_path_buf();
        let target = resolve_inside(output, &name)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        let mut file = File::create(target).map_err(io_error)?;
        std::io::copy(&mut entry, &mut file).map_err(io_error)?;
    }
    Ok(())
}

fn io_error(error: std::io::Error) -> NovelError {
    NovelError::Parse(format!("无法保存或读取 EPUB 数据：{error}"))
}
