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

export function listNovel(page: number, pageSize: number, order: NovelOrder) {
  return command<NovelSummary[]>("list_novels", {
    order,
    pageNumber: page,
    pageSize,
  });
}
export function rankNovels(days: number) {
  return command<NovelSummary[]>("rank_novels", { days });
}
export function searchByTags(query: string, page: number, pageSize: number) {
  return searchDiscovery(query, page, pageSize, "tags");
}
export function searchDiscovery(
  query: string,
  page: number,
  pageSize: number,
  mode: NovelSearchMode,
) {
  return command<DiscoveryList<NovelSummary>>("search_novels", {
    query,
    pageNumber: page,
    pageSize,
    mode,
  });
}
export function getReaderOverview(_source: string, bookId: string) {
  return command<NovelDetail>("get_reader_overview", { bookId });
}
export function getReaderDocument(
  _source: string,
  bookId: string,
  documentId: string,
  convert: "original" | "t2s" | "s2t" = "original",
) {
  return command<NovelChapterContent>("get_reader_document", {
    bookId,
    documentId,
    convert: convert === "original" ? null : convert,
  });
}
export function saveReadPosition(
  bookId: string,
  chapterId: string,
  xpath: string,
) {
  return command<void>("save_read_position", { bookId, chapterId, xpath });
}
