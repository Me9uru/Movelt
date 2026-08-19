import { ref } from "vue";
import type { NovelDetail } from "../services/novel";
import { addToBookshelf, listBookshelf, removeFromBookshelf, searchBookshelf, type BookshelfEntry } from "../services/library";

const books = ref<BookshelfEntry[]>([]);
export function useLibrary() {
  async function refreshBooks() { books.value = await listBookshelf(); }
  async function addBook(book: NovelDetail) { await addToBookshelf(book); await refreshBooks(); }
  async function removeBook(book: Pick<NovelDetail, "source" | "id">) { await removeFromBookshelf(book.source, book.id); await refreshBooks(); }
  return { books, refreshBooks, searchBooks: searchBookshelf, addBook, removeBook, isOnBookshelf: (book: Pick<NovelDetail, "id">) => books.value.some((entry) => entry.book.id === book.id) };
}
