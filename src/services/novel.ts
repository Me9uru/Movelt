import { command } from "./bridge";

export const lightNovelSourceId = "lightnovel";

export interface NovelSummary {
  source: string;
  id: string;
  title: string;
  cover_url: string | null;
  author: string | null;
  status: string | null;
  updated_at: string | null;
  description: string | null;
  tags: string[];
}
export interface NovelDetail extends NovelSummary {}
export interface ChapterSummary { id: string; title: string }
export interface Volume { title: string; chapters: ChapterSummary[]; sections: Volume[] }
export interface ServerReadPosition { chapterId: string; position: string }
export interface NovelOverview { detail: NovelDetail; volumes: Volume[]; readPosition: ServerReadPosition | null }
export interface ReaderDocument { id: string; bookId: string; chapterId: string; serverChapterId: string; title: string; html: string; fontUrl: string | null; readPosition: ServerReadPosition | null }
export interface DiscoveryList { items: NovelSummary[]; pagination: { page: number; previous: number | null; next: number | null; first: number; last: number } }
export type RankingSort = "latest" | "view" | "new";
export interface RecommendBlock { title: string; items: NovelSummary[] }

export function getLatest(page = 1) { return command<DiscoveryList>("get_latest", { pageNumber: page }); }
export function getRanking(sort: RankingSort, page = 1) { return command<DiscoveryList>("get_ranking", { sort, pageNumber: page }); }
export function getRank(days: number) { return command<NovelSummary[]>("get_rank", { days }); }
export function searchByTags(query: string, page = 1) { return command<DiscoveryList>("search_novels", { query, pageNumber: page, tags: true }); }
export function searchDiscovery(query: string, page = 1) { return command<DiscoveryList>("search_novels", { query, pageNumber: page, tags: false }); }
export function getReaderOverview(_source: string, bookId: string) { return command<NovelOverview>("get_reader_overview", { bookId }); }
export function getReaderDocument(_source: string, bookId: string, documentId: string) { return command<ReaderDocument>("get_reader_document", { bookId, documentId }); }
export function saveReadPosition(bookId: string, chapterId: string, xpath: string) { return command<void>("save_read_position", { bookId, chapterId, xpath }); }
