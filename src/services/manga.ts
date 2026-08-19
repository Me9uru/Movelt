import { invoke } from "@tauri-apps/api/core";

export interface MangaSummary { id: string; title: string; thumbnailUrl: string | null; author: string | null; unreadCount: number; sourceName: string | null; }
export interface MangaChapter { id: string; name: string; chapterNumber: number; isRead: boolean; lastPageRead: number; pageCount: number; }
export interface MangaDetail extends Omit<MangaSummary, "unreadCount"> { artist: string | null; description: string | null; genre: string[]; status: string; chapters: MangaChapter[]; }
export interface MangaPageList { chapterId: string; pageCount: number; firstPageUrls: string[]; }
export interface MangaPageBatch { startIndex: number; pageUrls: string[]; }

export type MangaBrowseType = "SEARCH" | "POPULAR" | "LATEST";
export function browseManga(query: string | null, page: number, browseType: MangaBrowseType): Promise<MangaSummary[]> { return invoke("browse_manga", { query, page, browseType }); }
export function getManga(mangaId: string): Promise<MangaDetail> { return invoke("get_manga", { mangaId }); }
export function getMangaChapterPages(mangaId: string, chapterId: string): Promise<MangaPageList> { return invoke("get_manga_chapter_pages", { mangaId, chapterId }); }
export function getMangaPageBatch(chapterId: string, pageIndex: number): Promise<MangaPageBatch> { return invoke("get_manga_page_batch", { chapterId, pageIndex }); }
