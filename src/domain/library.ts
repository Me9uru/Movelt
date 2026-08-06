import type { NovelDetail } from "../services/novel";

export interface ReadingProgress {
  documentId: string;
  documentTitle: string;
  location: number;
  bookLocation: number;
  updatedAt: string;
}

export interface BookshelfEntry {
  book: NovelDetail;
  addedAt: string;
  progress: ReadingProgress | null;
}
