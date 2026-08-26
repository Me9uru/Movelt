import { command } from "./bridge";
import type {
  DiscoveryList,
  NovelOverview,
  NovelSummary,
  ReaderDocument,
} from "../domain/novel";
import type { BookSearchMode } from "../domain/search";

export const lightNovelSourceId = "lightnovel";

export type RankingSort = "latest" | "view" | "new";
export type { BookSearchMode } from "../domain/search";

export function getLatest(page = 1) {
  return command<DiscoveryList<NovelSummary>>("get_latest", {
    pageNumber: page,
  });
}
export function getRanking(sort: RankingSort, page = 1) {
  return command<DiscoveryList<NovelSummary>>("get_ranking", {
    sort,
    pageNumber: page,
  });
}
export function getRank(days: number) {
  return command<NovelSummary[]>("get_rank", { days });
}
export function searchByTags(query: string, page = 1) {
  return searchDiscovery(query, page, "tags");
}
export function searchDiscovery(
  query: string,
  page = 1,
  mode: BookSearchMode = "title",
) {
  return command<DiscoveryList<NovelSummary>>("search_novels", {
    query,
    pageNumber: page,
    mode,
  });
}
export function getReaderOverview(_source: string, bookId: string) {
  return command<NovelOverview>("get_reader_overview", { bookId });
}
export function getReaderDocument(
  _source: string,
  bookId: string,
  documentId: string,
  convert: "original" | "t2s" | "s2t" = "original",
) {
  return command<ReaderDocument>("get_reader_document", {
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
