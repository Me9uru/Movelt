import { invoke } from "@tauri-apps/api/core";
import type { ReaderDocument } from "../domain/reader";

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

export interface NovelDetail extends NovelSummary {
}

export type RankingSort = "allvisit" | "allvote" | "monthvisit" | "monthvote"
  | "weekvisit" | "weekvote" | "dayvisit" | "dayvote" | "postdate"
  | "lastupdate" | "goodnum" | "size" | "fullflag" | "anime";
export interface PageInfo { page: number; previous: number | null; next: number | null; first: number; last: number; }
export interface DiscoveryList { items: NovelSummary[]; pagination: PageInfo; }
export interface RecommendBlock { title: string; items: NovelSummary[]; }
export interface HealthStatus { logged_in: boolean; base_url: string; }

export interface Volume {
  title: string;
  chapters: ChapterSummary[];
  sections: Volume[];
}

export interface NovelOverview {
  detail: NovelDetail;
  volumes: Volume[];
}

export interface ChapterSummary {
  id: string;
  title: string;
}

export type ChapterNode =
  | { type: "paragraph"; text: string }
  | { type: "image"; url: string; alt: string | null };

export function getReaderOverview(source: string, bookId: string): Promise<NovelOverview> {
  return invoke("get_reader_overview", { source, bookId });
}

const coverDataUrlCache = new Map<string, Promise<string>>();

export function getReaderCoverDataUrl(source: string, bookId: string): Promise<string> {
  const key = `${source}:${bookId}`;
  const cached = coverDataUrlCache.get(key);
  if (cached) {
    return cached;
  }
  const request = invoke<string>("get_reader_cover_data_url", { source, bookId })
    .catch((error) => {
      coverDataUrlCache.delete(key);
      throw error;
    });
  coverDataUrlCache.set(key, request);
  return request;
}

export function getReaderDocument(
  source: string,
  bookId: string,
  documentId: string,
  documentTitle?: string,
): Promise<ReaderDocument> {
  return invoke("get_reader_document", { source, bookId, documentId, documentTitle });
}

export function getDiscoveryHealth(): Promise<HealthStatus> {
  return invoke("discovery_health");
}
export function getRecommendations(): Promise<RecommendBlock[]> {
  return invoke("get_recommendations");
}
export function getRanking(sort: RankingSort, page = 1): Promise<DiscoveryList> {
  return invoke("get_ranking", { sort, page });
}
export function getCategory(tag: string, sort: RankingSort, page = 1): Promise<DiscoveryList> {
  return invoke("get_category", { tag, sort, page });
}
export function searchDiscovery(query: string, page = 1): Promise<DiscoveryList> {
  return invoke("search_discovery", { query, page });
}
