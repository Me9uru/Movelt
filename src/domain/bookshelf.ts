import type { NovelSummary } from "./novel";
import type { ComicSummary } from "./comic";

/** A novel shelf record returned by the application boundary. */
export interface NovelBookshelfEntry {
  book: NovelSummary;
  addedAt: string;
  progress: number | null;
}

/** A comic shelf record returned by the application boundary. */
export interface ComicBookshelfEntry {
  comic: ComicSummary;
  addedAt: string;
  progress: number | null;
}
