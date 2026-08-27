/**
 * Novel-domain DTOs rendered by the frontend.
 *
 * Novel and comic remain separate domains, but their summaries, catalogue
 * entries, reading positions, and reader payloads live here instead of in
 * transport adapters.
 */
import type { ReadPosition } from "./readPosition";

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

export interface NovelChapterSummary {
  id: string;
  title: string;
  sequence: number;
}

export interface NovelDetail extends NovelSummary {
  chapters: NovelChapterSummary[];
  readPosition: ReadPosition | null;
}

/** Sanitized novel chapter payload consumed by the text reader. */
export interface NovelChapterContent {
  chapterId: string;
  serverChapterId: string;
  title: string;
  html: string;
  fontUrl: string | null;
  readPosition: ReadPosition | null;
}

export type { DiscoveryList, RecommendBlock } from "./discovery";
