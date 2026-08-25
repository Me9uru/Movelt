import { ref } from "vue";
import type { NovelSummary } from "../domain/novel";
import { addToBookshelf, listBookshelf, removeFromBookshelf, searchBookshelf } from "../services/bookshelf";
import type { BookshelfEntry } from "../domain/bookshelf";

const books = ref<BookshelfEntry[]>([]);
export function useLibrary() {
  async function refreshBooks() { books.value = await listBookshelf(); }
  async function addBook(book: NovelSummary) { await addToBookshelf(book); await refreshBooks(); }
  async function removeBook(book: Pick<NovelSummary, "source" | "id">) { await removeFromBookshelf(book.source, book.id); await refreshBooks(); }
  return { books, refreshBooks, searchBooks: searchBookshelf, addBook, removeBook, isOnBookshelf: (book: Pick<NovelSummary, "id">) => books.value.some((entry) => entry.book.id === book.id) };
}
