import { command } from "./bridge";
import type {
  ComicBook,
  ComicSeriesDetail,
  ComicChapterPageBatch,
  ComicSummary,
} from "../domain/comic";
import type { DiscoveryList } from "../domain/discovery";
export type ComicOrder = "latest" | "view" | "new";
export type ComicSearchMode =
  | "fuzzy"
  | "exact"
  | "title"
  | "author"
  | "name"
  | "tags";

export function listComic(page: number, pageSize: number, order: ComicOrder) {
  return command<ComicSummary[]>("list_comics", {
    pageNumber: page,
    pageSize,
    order,
  });
}

export function searchComic(
  query: string,
  page: number,
  pageSize: number,
  searchMode: ComicSearchMode,
) {
  return command<DiscoveryList<ComicSummary>>("search_comics", {
    query,
    pageNumber: page,
    pageSize,
    mode: searchMode,
  });
}
export function listComicBookshelf(query?: string) {
  return query
    ? command<ComicSummary[]>("list_comic_bookshelf", { query })
    : command<ComicSummary[]>("list_comic_bookshelf");
}
export function isOnComicBookshelf(comicId: string) {
  return command<boolean>("is_on_comic_bookshelf", { comicId });
}
export function addToComicBookshelf(comicId: string) {
  return command<void>("set_comic_bookshelf", { comicId, present: true });
}
export function removeFromComicBookshelf(comicId: string) {
  return command<void>("set_comic_bookshelf", { comicId, present: false });
}
export function getComicSeries(seriesTitle: string) {
  return command<ComicSeriesDetail>("get_comic_series", { seriesTitle });
}
export function getComicBook(bookId: string) {
  return command<ComicBook>("get_comic_book", { bookId });
}
export function getComicChapterPages(
  comicId: string,
  chapterId: string,
  pageIndex: number,
) {
  return command<ComicChapterPageBatch>("get_comic_chapter_pages", {
    comicId,
    chapterId,
    pageIndex,
  });
}
export function saveComicReadPosition(
  comicId: string,
  chapterId: string,
  page: number,
) {
  return command<void>("save_comic_read_position", {
    comicId,
    chapterId,
    page,
  });
}
