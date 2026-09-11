pub(crate) mod bookshelf;
pub(crate) mod comic;
pub(crate) mod novel;
pub(crate) mod user;
mod validation;

#[cfg(test)]
mod tests {
    use super::{comic, novel};

    fn future_size<A, B, C, D, F>(_: impl FnOnce(A, B, C, D) -> F) -> usize {
        std::mem::size_of::<F>()
    }

    #[test]
    fn reader_command_futures_fit_android_ipc_stack() {
        // Tauri wraps and moves these futures on JavaBridge before spawning them.
        // Keep headroom for those copies and the surrounding JNI call stack.
        for (name, size) in [
            ("novel", future_size(novel::get_reader_document)),
            ("comic", future_size(comic::get_comic_chapter_pages)),
        ] {
            assert!(size <= 16 * 1024, "{name} command future uses {size} bytes");
        }
    }
}
