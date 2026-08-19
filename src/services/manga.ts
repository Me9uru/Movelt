import { requestWithHub } from "./lightnovel";

export interface MangaSummary { id: string; title: string; thumbnailUrl: string | null; author: string | null; unreadCount: number; sourceName: string | null; }
export interface MangaChapter { id: string; name: string; chapterNumber: number; isRead: boolean; lastPageRead: number; pageCount: number; }
export interface MangaDetail extends Omit<MangaSummary, "unreadCount"> { artist: string | null; description: string | null; genre: string[]; status: string; chapters: MangaChapter[]; }
export interface MangaPageList { chapterId: string; pageCount: number; firstPageUrls: string[]; }
export interface MangaPageBatch { startIndex: number; pageUrls: string[]; }
type Comic = { Id: number; Title: string; Cover?: string; Author?: string; Introduction?: string; LastUpdatedChapter?: string; Extra?: { classification?: { tags?: string[] } }; Chapter?: { Id: number; Title: string; SortNum: number; PageCount?: number }[]; Chapters?: { Id: number; Title: string; SortNum: number; PageCount?: number }[] };
type List<T> = { Data: T[] };
const toSummary = (item: Comic): MangaSummary => ({ id: String(item.Id), title: item.Title, thumbnailUrl: item.Cover || null, author: item.Author || null, unreadCount: 0, sourceName: "LightNovelShelf" });
export async function browseManga(query: string | null, page: number, browseType: "SEARCH" | "POPULAR" | "LATEST"): Promise<MangaSummary[]> {
  const method = browseType === "SEARCH" ? "SearchComicSeries" : "GetComicList";
  const payload = browseType === "SEARCH" ? { KeyWords: query, Page: page, Size: 30 } : { Page: page, Size: 30, Order: browseType === "POPULAR" ? "view" : "latest" };
  return (await requestWithHub<List<Comic>>(method, payload)).Data.map(toSummary);
}
export async function getManga(mangaId: string): Promise<MangaDetail> {
  const response = await requestWithHub<{ Book: Comic }>("GetComicInfo", { Id: Number(mangaId) }); const book = response.Book; const chapters = book.Chapters ?? book.Chapter ?? [];
  return { ...toSummary(book), artist: null, description: book.Introduction || null, genre: book.Extra?.classification?.tags ?? [], status: book.LastUpdatedChapter || "", chapters: chapters.map((chapter) => ({ id: String(chapter.Id), name: chapter.Title, chapterNumber: chapter.SortNum, isRead: false, lastPageRead: 0, pageCount: chapter.PageCount ?? 0 })) };
}
export async function getMangaChapterPages(_mangaId: string, chapterId: string): Promise<MangaPageList> {
  const response = await requestWithHub<{ Chapter: { Total: number; Images: { Url: string }[] } }>("GetComicContent", { Cid: Number(chapterId), Skip: 0, Take: 12 });
  return { chapterId, pageCount: response.Chapter.Total, firstPageUrls: response.Chapter.Images.map((image) => image.Url) };
}
export async function getMangaPageBatch(chapterId: string, pageIndex: number): Promise<MangaPageBatch> {
  const startIndex = Math.floor(pageIndex / 12) * 12; const response = await requestWithHub<{ Chapter: { Images: { Url: string }[] } }>("GetComicContent", { Cid: Number(chapterId), Skip: startIndex, Take: 12 });
  return { startIndex, pageUrls: response.Chapter.Images.map((image) => image.Url) };
}
