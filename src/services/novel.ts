import { command } from "./bridge";
import type {
  NovelDetail,
  NovelSummary,
  NovelChapterContent,
} from "../domain/novel";
import type { BookOrder, BookSearchRequestMode, DiscoveryList } from "../domain/discovery";
import type { ReaderConvert } from "../types/reader";


export const listNovel = (page: number, pageSize: number, order: BookOrder) => {
  return command<NovelSummary[]>("list_novels", {
    order,
    pageNumber: page,
    pageSize,
  });
};
export const rankNovels = (days: number) => {
  return command<NovelSummary[]>("rank_novels", { days });
};
export const searchDiscovery = (
  query: string,
  page: number,
  pageSize: number,
  mode: BookSearchRequestMode,
) => {
  return command<DiscoveryList<NovelSummary>>("search_novels", {
    query,
    pageNumber: page,
    pageSize,
    mode,
  });
};
export const getReaderOverview = (bookId: string) => {
  return command<NovelDetail>("get_reader_overview", { bookId });
};
export const getReaderDocument = (
  bookId: string,
  documentId: string,
  convert: ReaderConvert = "original",
) => {
  return command<NovelChapterContent>("get_reader_document", {
    bookId,
    documentId,
    convert: convert === "original" ? null : convert,
  });
};
export const saveReadPosition = (
  bookId: string,
  chapterId: string,
  xpath: string,
) => {
  return command<void>("save_read_position", { bookId, chapterId, xpath });
};
