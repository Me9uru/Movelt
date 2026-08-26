import { command } from "./bridge";
import type {
  MangaDetail,
  MangaPageBatch,
  MangaPageList,
  MangaSummary,
} from "../domain/manga";
import type { BookSearchMode } from "../domain/search";

export type MangaBrowseType = "SEARCH" | "TAGS" | "POPULAR" | "LATEST" | "NEW";

export function browseManga(
  query: string | null,
  page: number,
  browseType: MangaBrowseType,
  searchMode?: BookSearchMode,
) {
  return command<MangaSummary[]>("browse_manga", {
    query,
    pageNumber: page,
    browseType,
    searchMode,
  });
}
export function listMangaBookshelf(query?: string) {
  return query
    ? command<MangaSummary[]>("list_manga_bookshelf", { query })
    : command<MangaSummary[]>("list_manga_bookshelf");
}
export function isOnMangaBookshelf(mangaId: string) {
  return command<boolean>("is_on_manga_bookshelf", { mangaId });
}
export function addToMangaBookshelf(mangaId: string) {
  return command<void>("set_manga_bookshelf", { mangaId, present: true });
}
export function removeFromMangaBookshelf(mangaId: string) {
  return command<void>("set_manga_bookshelf", { mangaId, present: false });
}
export function getManga(mangaId: string) {
  return command<MangaDetail>("get_manga", { mangaId });
}
export function getMangaChapterPages(mangaId: string, chapterId: string) {
  return command<MangaPageList>("get_manga_chapter_pages", {
    mangaId,
    chapterId,
  });
}
export function getMangaPageBatch(chapterId: string, pageIndex: number) {
  return command<MangaPageBatch>("get_manga_page_batch", {
    chapterId,
    pageIndex,
  });
}
export function saveMangaReadPosition(
  mangaId: string,
  chapterId: string,
  page: number,
) {
  return command<void>("save_manga_read_position", {
    mangaId,
    chapterId,
    page,
  });
}
