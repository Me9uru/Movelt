/**
 * Manga-domain DTOs rendered by the frontend.
 *
 * Novel and manga remain separate domains, but their summaries, catalogue
 * entries, reading positions, and reader payloads live here instead of in
 * transport adapters.
 */
import type { ReadPosition } from "./readPosition";

export interface MangaSummary {
  id: string;
  title: string;
  thumbnailUrl: string | null;
  author: string | null;
  unreadCount: number;
  sourceName: string | null;
}

export interface MangaChapter {
  id: string;
  name: string;
  chapterNumber: number;
  isRead: boolean;
  lastPageRead: number;
  pageCount: number;
}

export interface MangaDetail extends Omit<MangaSummary, "unreadCount"> {
  artist: string | null;
  description: string | null;
  genre: string[];
  status: string;
  readPosition: ReadPosition | null;
  chapters: MangaChapter[];
}

export interface MangaPageList {
  chapterId: string;
  pageCount: number;
  firstPageUrls: string[];
  readPosition: ReadPosition | null;
}

export interface MangaPageBatch {
  startIndex: number;
  pageUrls: string[];
}
