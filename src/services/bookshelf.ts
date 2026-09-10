import { command } from "./bridge";
import type { NovelBookshelfEntry } from "../domain/bookshelf";

export type { NovelBookshelfEntry } from "../domain/bookshelf";

export const listNovelBookshelf = () => {
  return command<NovelBookshelfEntry[]>("list_novel_bookshelf");
};

export const isOnNovelBookshelf = (bookId: string) => {
  return command<boolean>("is_on_novel_bookshelf", { bookId });
};

export const addToNovelBookshelf = (bookId: string) => {
  return command<void>("set_novel_bookshelf", {
    bookId,
    present: true,
  });
};

export const removeFromNovelBookshelf = (bookId: string) => {
  return command<void>("set_novel_bookshelf", { bookId, present: false });
};
