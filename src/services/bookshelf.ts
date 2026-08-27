import { command } from "./bridge";
import type { NovelSummary } from "../domain/novel";
import type { BookshelfEntry } from "../domain/bookshelf";

export type { BookshelfEntry } from "../domain/bookshelf";

export function listNovelBookshelf(query?: string) {
  return query
    ? command<BookshelfEntry[]>("list_novel_bookshelf", { query })
    : command<BookshelfEntry[]>("list_novel_bookshelf");
}

export function isOnNovelBookshelf(bookId: string) {
  return command<boolean>("is_on_novel_bookshelf", { bookId });
}

export function addToNovelBookshelf(book: NovelSummary) {
  return command<void>("set_novel_bookshelf", {
    bookId: book.id,
    present: true,
  });
}

export function removeFromNovelBookshelf(_source: string, bookId: string) {
  return command<void>("set_novel_bookshelf", { bookId, present: false });
}
