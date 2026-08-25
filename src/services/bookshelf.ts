import { command } from "./bridge";
import type { NovelSummary } from "../domain/novel";
import type { BookshelfEntry } from "../domain/bookshelf";

export type { BookshelfEntry } from "../domain/bookshelf";

export function listBookshelf() {
  return command<BookshelfEntry[]>("list_bookshelf");
}

export function searchBookshelf(query: string) {
  return command<BookshelfEntry[]>("list_bookshelf", { query });
}

export function addToBookshelf(book: NovelSummary) {
  return command<void>("set_novel_bookshelf", {
    bookId: book.id,
    present: true,
  });
}

export function removeFromBookshelf(_source: string, bookId: string) {
  return command<void>("set_novel_bookshelf", { bookId, present: false });
}
