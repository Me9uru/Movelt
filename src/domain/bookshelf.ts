import type { NovelSummary } from "./novel";

/** A novel shelf record returned by the application boundary. */
export interface BookshelfEntry {
  book: NovelSummary;
  addedAt: string;
  progress: null;
}
