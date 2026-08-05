import { ref } from "vue";
import type { NovelDetail } from "../services/novel";
import type { BookshelfEntry, ReadingProgress } from "../domain/library";
import {
  addToBookshelf,
  getReadingProgress,
  listBookshelf,
  removeFromBookshelf,
  saveReadingProgress,
} from "../services/library";

const books = ref<BookshelfEntry[]>([]);
const progress = ref<Record<string, ReadingProgress>>({});
let progressSaveQueue = Promise.resolve();

function keyFor(book: Pick<NovelDetail, "source" | "id">): string {
  return `${book.source}:${book.id}`;
}

export function useLibrary() {
  async function refreshBooks(): Promise<void> {
    const entries = await listBookshelf();
    books.value = entries;
    for (const entry of entries) {
      if (entry.progress) progress.value[keyFor(entry.book)] = entry.progress;
    }
  }

  function isOnBookshelf(book: Pick<NovelDetail, "source" | "id">): boolean {
    const key = keyFor(book);
    return books.value.some((entry) => keyFor(entry.book) === key);
  }

  async function addBook(book: NovelDetail): Promise<void> {
    await addToBookshelf(book);
    await refreshBooks();
  }

  async function removeBook(book: Pick<NovelDetail, "source" | "id">): Promise<void> {
    await removeFromBookshelf(book.source, book.id);
    await refreshBooks();
  }

  function progressFor(book: Pick<NovelDetail, "source" | "id">): ReadingProgress | null {
    return progress.value[keyFor(book)] ?? null;
  }

  async function loadProgress(
    book: Pick<NovelDetail, "source" | "id">,
  ): Promise<ReadingProgress | null> {
    const saved = await getReadingProgress(book.source, book.id);
    if (saved) progress.value[keyFor(book)] = saved;
    else delete progress.value[keyFor(book)];
    return saved;
  }

  function saveProgress(
    book: Pick<NovelDetail, "source" | "id">,
    value: Omit<ReadingProgress, "updatedAt">,
  ): Promise<ReadingProgress> {
    const request = progressSaveQueue.then(async () => {
      const saved = await saveReadingProgress(book.source, book.id, value);
      progress.value[keyFor(book)] = saved;
      const entry = books.value.find((item) => keyFor(item.book) === keyFor(book));
      if (entry) entry.progress = saved;
      books.value = [...books.value].sort((a, b) => {
        const aTime = a.progress?.updatedAt ?? a.addedAt;
        const bTime = b.progress?.updatedAt ?? b.addedAt;
        return bTime.localeCompare(aTime);
      });
      return saved;
    });
    progressSaveQueue = request.then(() => undefined, () => undefined);
    return request;
  }

  return {
    books,
    refreshBooks,
    addBook,
    removeBook,
    isOnBookshelf,
    progressFor,
    loadProgress,
    saveProgress,
  };
}
