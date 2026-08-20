import { command } from "./bridge";

export interface MangaSummary { id: string; title: string; thumbnailUrl: string | null; author: string | null; unreadCount: number; sourceName: string | null }
export interface MangaChapter { id: string; name: string; chapterNumber: number; isRead: boolean; lastPageRead: number; pageCount: number }
export interface MangaDetail extends Omit<MangaSummary, "unreadCount"> { artist: string | null; description: string | null; genre: string[]; status: string; chapters: MangaChapter[] }
export interface MangaPageList { chapterId: string; pageCount: number; firstPageUrls: string[] }
export interface MangaPageBatch { startIndex: number; pageUrls: string[] }
export type MangaBrowseType = "SEARCH" | "TAGS" | "POPULAR" | "LATEST" | "NEW";

export function browseManga(query: string | null, page: number, browseType: MangaBrowseType) { return command<MangaSummary[]>("browse_manga", { query, pageNumber: page, browseType }); }
export function listMangaBookshelf() { return command<MangaSummary[]>("list_manga_bookshelf"); }
export function isOnMangaBookshelf(mangaId: string) { return command<boolean>("is_on_manga_bookshelf", { mangaId }); }
export function addToMangaBookshelf(mangaId: string) { return command<void>("set_manga_bookshelf", { mangaId, present: true }); }
export function removeFromMangaBookshelf(mangaId: string) { return command<void>("set_manga_bookshelf", { mangaId, present: false }); }
export function getManga(mangaId: string) { return command<MangaDetail>("get_manga", { mangaId }); }
export function getMangaChapterPages(_mangaId: string, chapterId: string) { return command<MangaPageList>("get_manga_chapter_pages", { chapterId }); }
export function getMangaPageBatch(chapterId: string, pageIndex: number) { return command<MangaPageBatch>("get_manga_page_batch", { chapterId, pageIndex }); }
