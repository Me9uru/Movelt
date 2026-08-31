import { ref } from "vue";
import { defineStore } from "pinia";
import type { NovelSummary } from "../domain/novel";
import type { ComicSummary } from "../domain/comic";
import {
  addToNovelBookshelf,
  listNovelBookshelf,
  removeFromNovelBookshelf,
} from "../services/bookshelf";
import {
  addToComicBookshelf,
  listComicBookshelf,
  removeFromComicBookshelf,
} from "../services/comic";
import type {
  ComicBookshelfEntry,
  NovelBookshelfEntry,
} from "../domain/bookshelf";

/**
 * 会话级书架缓存。操作成功后直接更新缓存，避免重新请求整个官方书架。
 */
export const useBookshelfStore = defineStore("bookshelf", () => {
  const books = ref<NovelBookshelfEntry[]>([]);
  const comic = ref<ComicBookshelfEntry[]>([]);
  const novelBooksLoaded = ref(false);
  const comicBooksLoaded = ref(false);
  const addedNovelIds = ref(new Set<string>());
  const removedNovelIds = ref(new Set<string>());

  const refreshBooks = async (): Promise<void> => {
    books.value = await listNovelBookshelf();
    novelBooksLoaded.value = true;
    addedNovelIds.value.clear();
    removedNovelIds.value.clear();
  }

  const refreshComicBooks = async (): Promise<void> => {
    comic.value = await listComicBookshelf();
    comicBooksLoaded.value = true;
  }

  const addBook = async (book: NovelSummary): Promise<void> => {
    await addToNovelBookshelf(book);
    addedNovelIds.value.add(book.id);
    removedNovelIds.value.delete(book.id);
    if (!novelBooksLoaded.value || books.value.some((entry) => entry.book.id === book.id)) return;
    books.value.unshift({ book, addedAt: new Date().toISOString(), progress: null });
  }

  const removeBook = async (book: Pick<NovelSummary, "source" | "id">): Promise<void> => {
    await removeFromNovelBookshelf(book.source, book.id);
    addedNovelIds.value.delete(book.id);
    removedNovelIds.value.add(book.id);
    if (novelBooksLoaded.value) {
      books.value = books.value.filter((entry) => entry.book.id !== book.id);
    }
  }

  const addComicBook = async (item: ComicSummary): Promise<void> => {
    await addToComicBookshelf(item.id);
    if (
      !comicBooksLoaded.value ||
      comic.value.some((entry) => entry.comic.id === item.id)
    ) return;
    comic.value.unshift({
      comic: item,
      addedAt: new Date().toISOString(),
      progress: null,
    });
  }

  const removeComicBook = async (comicId: string): Promise<void> => {
    await removeFromComicBookshelf(comicId);
    if (comicBooksLoaded.value) {
      comic.value = comic.value.filter((entry) => entry.comic.id !== comicId);
    }
  }

  const searchBooks = (query: string): NovelBookshelfEntry[] => {
    const normalized = query.toLowerCase();
    return books.value.filter((entry) => entry.book.title.toLowerCase().includes(normalized));
  }

  const searchComicBooks = (query: string): ComicBookshelfEntry[] => {
    const normalized = query.toLowerCase();
    return comic.value.filter((entry) =>
      entry.comic.title.toLowerCase().includes(normalized),
    );
  }

  const isOnBookshelf = (book: Pick<NovelSummary, "id">): boolean => {
    return addedNovelIds.value.has(book.id) ||
      (!removedNovelIds.value.has(book.id) &&
        books.value.some((entry) => entry.book.id === book.id));
  }

  const clear = (): void => {
    books.value = [];
    comic.value = [];
    novelBooksLoaded.value = false;
    comicBooksLoaded.value = false;
    addedNovelIds.value.clear();
    removedNovelIds.value.clear();
  }

  return {
    books,
    comic,
    novelBooksLoaded,
    comicBooksLoaded,
    addedNovelIds,
    removedNovelIds,
    refreshBooks,
    refreshComicBooks,
    searchBooks,
    searchComicBooks,
    addBook,
    removeBook,
    addComicBook,
    removeComicBook,
    isOnBookshelf,
    clear,
  };
});
