import { command } from "./bridge";
import type {
  DiscoveryList,
  NovelDetail,
  NovelSummary,
  NovelChapterContent,
} from "../domain/novel";
export const lightNovelSourceId = "lightnovel";

export type NovelOrder = "latest" | "view" | "new";
export type NovelSearchMode =
  | "fuzzy"
  | "exact"
  | "title"
  | "author"
  | "name"
  | "tags";
export type { BookSearchMode } from "../domain/search";

export const listNovel = (page: number, pageSize: number, order: NovelOrder) => {
  return command<NovelSummary[]>("list_novels", {
    order,
    pageNumber: page,
    pageSize,
  });
}
export const rankNovels = (days: number) => {
  return command<NovelSummary[]>("rank_novels", { days });
}
export const searchByTags = (query: string, page: number, pageSize: number) => {
  return searchDiscovery(query, page, pageSize, "tags");
}
export const searchDiscovery = (
  query: string,
  page: number,
  pageSize: number,
  mode: NovelSearchMode,
) => {
  return command<DiscoveryList<NovelSummary>>("search_novels", {
    query,
    pageNumber: page,
    pageSize,
    mode,
  });
}
export const getReaderOverview = (_source: string, bookId: string) => {
  return command<NovelDetail>("get_reader_overview", { bookId });
}
export const getReaderDocument = (
  _source: string,
  bookId: string,
  documentId: string,
  convert: "original" | "t2s" | "s2t" = "original",
) => {
  return command<NovelChapterContent>("get_reader_document", {
    bookId,
    documentId,
    convert: convert === "original" ? null : convert,
  });
}
export const saveReadPosition = (
  bookId: string,
  chapterId: string,
  xpath: string,
) => {
  return command<void>("save_read_position", { bookId, chapterId, xpath });
}
