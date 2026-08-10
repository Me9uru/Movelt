import { invoke } from "@tauri-apps/api/core";

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

export interface ChapterContent {
  source: string;
  novel_id: string;
  chapter_id: string;
  title: string;
  nodes: ChapterNode[];
}

export type ChapterNode =
  | { type: "paragraph"; text: string }
  | { type: "image"; url: string; alt: string | null };

export function getNovelOverview(source: string, novelId: string): Promise<NovelOverview> {
  return invoke("get_novel_overview", { source, novelId });
}

const coverDataUrlCache = new Map<string, Promise<string>>();

export function getNovelCoverDataUrl(source: string, novelId: string): Promise<string> {
  const key = `${source}:${novelId}`;
  const cached = coverDataUrlCache.get(key);
  if (cached) {
    return cached;
  }
  const request = invoke<string>("get_novel_cover_data_url", { source, novelId })
    .catch((error) => {
      coverDataUrlCache.delete(key);
      throw error;
    });
  coverDataUrlCache.set(key, request);
  return request;
}

export function getChapter(
  source: string,
  novelId: string,
  chapterId: string,
  chapterTitle?: string,
): Promise<ChapterContent> {
  return invoke("get_chapter", { source, novelId, chapterId, chapterTitle });
}

export function prefetchChapters(
  source: string,
  novelId: string,
  chapterIds: string[],
  chapterTitles?: string[],
): Promise<void> {
  return invoke("prefetch_chapters", { source, novelId, chapterIds, chapterTitles });
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
