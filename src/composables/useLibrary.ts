import { ref } from "vue";
import type { NovelSummary } from "../domain/novel";
import type { ComicSummary } from "../domain/comic";
import {
  addToBookshelf,
  listBookshelf,
  removeFromBookshelf,
} from "../services/bookshelf";
import {
  addToComicBookshelf,
  listComicBookshelf,
  removeFromComicBookshelf,
} from "../services/comic";
import type { BookshelfEntry } from "../domain/bookshelf";

const books = ref<BookshelfEntry[]>([]);
const comic = ref<ComicSummary[]>([]);
let novelBooksLoaded = false;
let comicBooksLoaded = false;
const addedNovelIds = new Set<string>();
const removedNovelIds = new Set<string>();

/**
 * 会话级书架缓存。操作成功后直接更新缓存，避免重新请求整个官方书架。
 */
export function useLibrary() {
  async function refreshBooks(): Promise<void> {
    books.value = await listBookshelf();
    novelBooksLoaded = true;
    addedNovelIds.clear();
    removedNovelIds.clear();
  }

  async function refreshComicBooks(): Promise<void> {
    comic.value = await listComicBookshelf();
    comicBooksLoaded = true;
  }

  async function addBook(book: NovelSummary): Promise<void> {
    await addToBookshelf(book);
    addedNovelIds.add(book.id);
    removedNovelIds.delete(book.id);
    if (!novelBooksLoaded || books.value.some((entry) => entry.book.id === book.id)) return;
    books.value.unshift({ book, addedAt: new Date().toISOString(), progress: null });
  }

  async function removeBook(book: Pick<NovelSummary, "source" | "id">): Promise<void> {
    await removeFromBookshelf(book.source, book.id);
    addedNovelIds.delete(book.id);
    removedNovelIds.add(book.id);
    if (novelBooksLoaded) {
      books.value = books.value.filter((entry) => entry.book.id !== book.id);
    }
  }

  async function addComicBook(item: ComicSummary): Promise<void> {
    await addToComicBookshelf(item.id);
    if (!comicBooksLoaded || comic.value.some((comicItem) => comicItem.id === item.id)) return;
    comic.value.unshift(item);
  }

  async function removeComicBook(comicId: string): Promise<void> {
    await removeFromComicBookshelf(comicId);
    if (comicBooksLoaded) comic.value = comic.value.filter((item) => item.id !== comicId);
  }

  function searchBooks(query: string): BookshelfEntry[] {
    const normalized = query.toLowerCase();
    return books.value.filter((entry) => entry.book.title.toLowerCase().includes(normalized));
  }

  function searchComicBooks(query: string): ComicSummary[] {
    const normalized = query.toLowerCase();
    return comic.value.filter((item) => item.title.toLowerCase().includes(normalized));
  }

  return {
    books,
    comic,
    refreshBooks,
    refreshComicBooks,
    searchBooks,
    searchComicBooks,
    addBook,
    removeBook,
    addComicBook,
    removeComicBook,
    isOnBookshelf: (book: Pick<NovelSummary, "id">) =>
      addedNovelIds.has(book.id) ||
      (!removedNovelIds.has(book.id) &&
        books.value.some((entry) => entry.book.id === book.id)),
  };
}

/** 登录态切换时清空所有会话级书架数据。 */
export function clearLibraryCache(): void {
  books.value = [];
  comic.value = [];
  novelBooksLoaded = false;
  comicBooksLoaded = false;
  addedNovelIds.clear();
  removedNovelIds.clear();
}
