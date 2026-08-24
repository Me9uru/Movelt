/**
 * Content-domain DTOs rendered by the frontend.
 *
 * Novel and manga remain separate domains, but their summaries, catalogue
 * entries, reading positions, and reader payloads live here instead of in
 * transport adapters.
 */

export interface ReadPosition {
  chapterId: string;
  position: string;
}

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

export interface ChapterSummary {
  id: string;
  title: string;
}

export interface Volume {
  title: string;
  chapters: ChapterSummary[];
  sections: Volume[];
}

export interface NovelOverview {
  detail: NovelSummary;
  volumes: Volume[];
  readPosition: ReadPosition | null;
}

/** Sanitized novel chapter payload consumed by the text reader. */
export interface ReaderDocument {
  id: string;
  bookId: string;
  chapterId: string;
  serverChapterId: string;
  title: string;
  html: string;
  fontUrl: string | null;
  readPosition: ReadPosition | null;
}

export interface DiscoveryList {
  items: NovelSummary[];
  pagination: {
    page: number;
    previous: number | null;
    next: number | null;
    first: number;
    last: number;
  };
}

export interface RecommendBlock {
  title: string;
  items: NovelSummary[];
}

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
