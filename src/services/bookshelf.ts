import { command } from "./bridge";
import type { NovelSummary } from "../domain/novel";
import type { NovelBookshelfEntry } from "../domain/bookshelf";

export type { NovelBookshelfEntry } from "../domain/bookshelf";

export const listNovelBookshelf = (query?: string) => {
  return query
    ? command<NovelBookshelfEntry[]>("list_novel_bookshelf", { query })
    : command<NovelBookshelfEntry[]>("list_novel_bookshelf");
}

export const isOnNovelBookshelf = (bookId: string) => {
  return command<boolean>("is_on_novel_bookshelf", { bookId });
}

export const addToNovelBookshelf = (book: NovelSummary) => {
  return command<void>("set_novel_bookshelf", {
    bookId: book.id,
    present: true,
  });
}

export const removeFromNovelBookshelf = (_source: string, bookId: string) => {
  return command<void>("set_novel_bookshelf", { bookId, present: false });
}
