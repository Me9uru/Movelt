import { command } from "./bridge";
import type { NovelDetail, NovelSummary } from "./novel";

export interface BookshelfEntry { book: NovelSummary; addedAt: string; progress: null }

export function listBookshelf() { return command<BookshelfEntry[]>("list_bookshelf"); }
export function searchBookshelf(query: string) { return command<BookshelfEntry[]>("list_bookshelf", { query }); }
export function addToBookshelf(book: NovelDetail) { return command<void>("set_novel_bookshelf", { bookId: book.id, present: true }); }
export function removeFromBookshelf(_source: string, bookId: string) { return command<void>("set_novel_bookshelf", { bookId, present: false }); }
