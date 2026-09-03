import { command } from "./bridge";
import type {
  ComicBook,
  ComicSeriesDetail,
  ComicChapterPageBatch,
  ComicSummary,
} from "../domain/comic";
import type { DiscoveryList } from "../domain/discovery";
import type { ComicBookshelfEntry } from "../domain/bookshelf";
export type ComicOrder = "latest" | "view" | "new";
export type ComicSearchMode =
  | "fuzzy"
  | "exact"
  | "title"
  | "author"
  | "name"
  | "tags";

export const listComic = (page: number, pageSize: number, order: ComicOrder) => {
  return command<ComicSummary[]>("list_comics", {
    pageNumber: page,
    pageSize,
    order,
  });
}

export const searchComic = (
  query: string,
  page: number,
  pageSize: number,
  searchMode: ComicSearchMode,
) => {
  return command<DiscoveryList<ComicSummary>>("search_comics", {
    query,
    pageNumber: page,
    pageSize,
    mode: searchMode,
  });
}
export const listComicBookshelf = () => {
  return command<ComicBookshelfEntry[]>("list_comic_bookshelf");
}
export const isOnComicBookshelf = (comicId: string) => {
  return command<boolean>("is_on_comic_bookshelf", { comicId });
}
export const addToComicBookshelf = (comicId: string) => {
  return command<void>("set_comic_bookshelf", { comicId, present: true });
}
export const removeFromComicBookshelf = (comicId: string) => {
  return command<void>("set_comic_bookshelf", { comicId, present: false });
}
export const getComicSeries = (seriesTitle: string) => {
  return command<ComicSeriesDetail>("get_comic_series", { seriesTitle });
}
export const getComicBook = (bookId: string) => {
  return command<ComicBook>("get_comic_book", { bookId });
}
export const getComicChapterPages = (
  comicId: string,
  chapterId: string,
  pageIndex: number,
) => {
  return command<ComicChapterPageBatch>("get_comic_chapter_pages", {
    comicId,
    chapterId,
    pageIndex,
  });
}
export const saveComicReadPosition = (
  comicId: string,
  chapterId: string,
  page: number,
) => {
  return command<void>("save_comic_read_position", {
    comicId,
    chapterId,
    page,
  });
}
