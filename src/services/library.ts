import { requestWithHub } from "./lightnovel";
import { getBooksByIds, type NovelDetail, type NovelSummary } from "./novel";

export interface BookshelfEntry { book: NovelSummary; addedAt: string; progress: null; }
type ShelfItem = { id: number; type: "BOOK" | "book" | number; parents?: string[]; index?: number; updateAt?: string };
type RemoteShelf = { data?: ShelfItem[]; ver?: string };

async function shelf(): Promise<RemoteShelf> { return requestWithHub<RemoteShelf>("GetBookShelf"); }
async function save(data: ShelfItem[], ver?: string): Promise<void> { await requestWithHub("SaveBookShelf", { data, ver: ver ?? "20220211" }); }

export async function listBookshelf(): Promise<BookshelfEntry[]> {
  const remote = await shelf();
  const items = (remote.data ?? []).filter((item) => item.type === "BOOK" || item.type === "book" || item.type === 0);
  const books = await getBooksByIds(items.map((item) => item.id));
  const byId = new Map(books.map((book) => [Number(book.id), book]));
  return items.map((item) => ({ book: byId.get(item.id), addedAt: item.updateAt ?? "" , progress: null })).filter((entry): entry is BookshelfEntry => Boolean(entry.book));
}
export async function searchBookshelf(query: string): Promise<BookshelfEntry[]> {
  const needle = query.trim().toLowerCase();
  return (await listBookshelf()).filter((entry) => !needle || entry.book.title.toLowerCase().includes(needle));
}
export async function addToBookshelf(book: NovelDetail): Promise<void> {
  const remote = await shelf(); const data = remote.data ?? [];
  if (!data.some((item) => item.id === Number(book.id))) data.unshift({ id: Number(book.id), type: "BOOK", parents: [], index: 0, updateAt: new Date().toISOString() });
  await save(data, remote.ver);
}
export async function removeFromBookshelf(_source: string, bookId: string): Promise<void> {
  const remote = await shelf(); await save((remote.data ?? []).filter((item) => item.id !== Number(bookId)), remote.ver);
}
