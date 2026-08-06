import { invoke } from "@tauri-apps/api/core";

export interface SearchResult {
  page: number;
  total_pages: number;
  items: NovelSummary[];
}

export interface NovelSourceInfo {
  id: string;
  name: string;
}

export interface NovelSummary {
  source: string;
  id: string;
  title: string;
  cover_url: string | null;
}

export interface NovelDetail extends NovelSummary {
  author: string | null;
  status: string | null;
  updated_at: string | null;
  description: string | null;
}

export interface Volume {
  title: string;
  chapters: ChapterSummary[];
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

export function listNovelSources(): Promise<NovelSourceInfo[]> {
  return invoke("list_novel_sources");
}

export function searchNovels(source: string, query: string, page = 1): Promise<SearchResult> {
  return invoke("search_novels", { source, query, page });
}

export function getNovelOverview(source: string, novelId: string): Promise<NovelOverview> {
  return invoke("get_novel_overview", { source, novelId });
}

export function getChapter(
  source: string,
  novelId: string,
  chapterId: string,
): Promise<ChapterContent> {
  return invoke("get_chapter", { source, novelId, chapterId });
}

export function prefetchChapters(
  source: string,
  novelId: string,
  chapterIds: string[],
): Promise<void> {
  return invoke("prefetch_chapters", { source, novelId, chapterIds });
}
