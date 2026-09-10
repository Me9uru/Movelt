import { ref } from "vue";
import { defineStore } from "pinia";
import type { NovelSummary } from "../domain/novel";
import type { ComicSummary } from "../domain/comic";
import {
  addToNovelBookshelf,
  isOnNovelBookshelf,
  listNovelBookshelf,
  removeFromNovelBookshelf,
} from "../services/bookshelf";
import {
  addToComicBookshelf,
  isOnComicBookshelf,
  listComicBookshelf,
  removeFromComicBookshelf,
} from "../services/comic";
import type { ComicBookshelfEntry, NovelBookshelfEntry } from "../domain/bookshelf";
import { fuzzyIncludes } from "../utils/fuzzySearch";

/** Session cache: membership is unknown until a list or an explicit query resolves. */
export const useBookshelfStore = defineStore("bookshelf", () => {
  const books = ref<NovelBookshelfEntry[]>([]);
  const comic = ref<ComicBookshelfEntry[]>([]);
  const novelBooksLoaded = ref(false);
  const comicBooksLoaded = ref(false);
  const novelMembership = ref(new Map<string, boolean>());
  const comicMembership = ref(new Map<string, boolean>());
  let session = 0;
  let novelRevision = 0;
  let comicRevision = 0;

  const refreshBooks = async (): Promise<void> => {
    const revision = ++novelRevision;
    const result = await listNovelBookshelf();
    if (revision !== novelRevision) return;
    books.value = result;
    novelBooksLoaded.value = true;
    novelMembership.value.clear();
  };

  const refreshComicBooks = async (): Promise<void> => {
    const revision = ++comicRevision;
    const result = await listComicBookshelf();
    if (revision !== comicRevision) return;
    comic.value = result;
    comicBooksLoaded.value = true;
    comicMembership.value.clear();
  };

  const isOnBookshelf = (book: Pick<NovelSummary, "id">): boolean | null =>
    novelMembership.value.get(book.id) ??
    (novelBooksLoaded.value ? books.value.some((entry) => entry.book.id === book.id) : null);

  const isComicOnBookshelf = (bookId: string): boolean | null =>
    comicMembership.value.get(bookId) ??
    (comic.value.some((entry) => entry.comic.bookId === bookId) ? true : null);

  const ensureNovelMembership = async (bookId: string): Promise<void> => {
    if (isOnBookshelf({ id: bookId }) !== null) return;
    const revision = novelRevision;
    const present = await isOnNovelBookshelf(bookId);
    if (revision === novelRevision) novelMembership.value.set(bookId, present);
  };

  const ensureComicMembership = async (bookId: string): Promise<void> => {
    if (isComicOnBookshelf(bookId) !== null) return;
    const revision = comicRevision;
    const present = await isOnComicBookshelf(bookId);
    if (revision === comicRevision) comicMembership.value.set(bookId, present);
  };

  const addBook = async (book: NovelSummary): Promise<void> => {
    const owner = session;
    await addToNovelBookshelf(book.id);
    if (owner !== session) return;
    novelRevision += 1;
    novelMembership.value.set(book.id, true);
    if (!novelBooksLoaded.value || books.value.some((entry) => entry.book.id === book.id)) return;
    books.value.unshift({ book, addedAt: new Date().toISOString(), progress: null });
  };

  const removeBook = async (book: Pick<NovelSummary, "id">): Promise<void> => {
    const owner = session;
    await removeFromNovelBookshelf(book.id);
    if (owner !== session) return;
    novelRevision += 1;
    novelMembership.value.set(book.id, false);
    books.value = books.value.filter((entry) => entry.book.id !== book.id);
  };

  const addComicBook = async (item: ComicSummary, bookId: string): Promise<void> => {
    const owner = session;
    await addToComicBookshelf(bookId);
    if (owner !== session) return;
    comicRevision += 1;
    comicMembership.value.set(bookId, true);
    if (!comicBooksLoaded.value || comic.value.some((entry) => entry.comic.bookId === bookId)) return;
    comic.value.unshift({
      comic: { id: item.id, bookId, title: item.title, coverUrl: item.coverUrl, author: item.author },
      addedAt: new Date().toISOString(),
      progress: null,
    });
  };

  const removeComicBook = async (bookId: string): Promise<void> => {
    const owner = session;
    await removeFromComicBookshelf(bookId);
    if (owner !== session) return;
    comicRevision += 1;
    comicMembership.value.set(bookId, false);
    comic.value = comic.value.filter((entry) => entry.comic.bookId !== bookId);
  };

  const searchBooks = (query: string): NovelBookshelfEntry[] =>
    books.value.filter((entry) => fuzzyIncludes(entry.book.title, query));

  const searchComicBooks = (query: string): ComicBookshelfEntry[] =>
    comic.value.filter((entry) => fuzzyIncludes(entry.comic.title, query));

  const clear = (): void => {
    session += 1;
    novelRevision += 1;
    comicRevision += 1;
    books.value = [];
    comic.value = [];
    novelBooksLoaded.value = false;
    comicBooksLoaded.value = false;
    novelMembership.value.clear();
    comicMembership.value.clear();
  };

  return {
    books, comic, novelBooksLoaded, comicBooksLoaded,
    refreshBooks, refreshComicBooks, searchBooks, searchComicBooks,
    addBook, removeBook, addComicBook, removeComicBook,
    isOnBookshelf, isComicOnBookshelf, ensureNovelMembership, ensureComicMembership, clear,
  };
});
