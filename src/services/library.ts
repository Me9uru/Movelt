import { invoke } from "@tauri-apps/api/core";
import type { BookshelfEntry, ReadingProgress } from "../domain/library";
import type { NovelDetail } from "./novel";

export function listBookshelf(): Promise<BookshelfEntry[]> {
  return invoke("list_bookshelf");
}

export function searchBookshelf(query: string): Promise<BookshelfEntry[]> {
  return invoke("search_bookshelf", { query });
}

export function addToBookshelf(book: NovelDetail): Promise<void> {
  return invoke("add_to_bookshelf", { book });
}

export function removeFromBookshelf(source: string, bookId: string): Promise<void> {
  return invoke("remove_from_bookshelf", { source, bookId });
}

export function getReadingProgress(
  source: string,
  bookId: string,
): Promise<ReadingProgress | null> {
  return invoke("get_reading_progress", { source, bookId });
}

export function saveReadingProgress(
  source: string,
  bookId: string,
  progress: Omit<ReadingProgress, "updatedAt">,
): Promise<ReadingProgress> {
  return invoke("save_reading_progress", { source, bookId, progress });
}
