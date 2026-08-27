/**
 * Comic-domain DTOs rendered by the frontend.
 *
 * Novel and comic remain separate domains, but their summaries, catalogue
 * entries, reading positions, and reader payloads live here instead of in
 * transport adapters.
 */
import type { ReadPosition } from "./readPosition";

export interface ComicSummary {
  id: string;
  /** Official numeric volume ID, populated only for bookshelf entries. */
  bookId?: string | null;
  title: string;
  coverUrl: string | null;
  author: string | null;
}

export interface ComicChapterSummary {
  id: string;
  title: string;
  sequence: number;
  pageCount: number;
}

export interface ComicBook {
  id: string;
  title: string;
  readPosition: ReadPosition | null;
  chapters: ComicChapterSummary[];
}

export interface ComicSeriesDetail extends ComicSummary {
  description: string | null;
  genre: string[];
  status: string;
  books: ComicBook[];
}

export interface ComicChapterPageBatch {
  chapterId: string;
  startIndex: number;
  pageCount: number;
  pageUrls: string[];
  readPosition: ReadPosition | null;
}
