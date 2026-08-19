import DOMPurify from "dompurify";

import { requestWithHub } from "./lightnovel";

export const lightNovelSourceId = "lightnovel";

type ListResult<T> = { TotalPages: number; Page: number; Data: T[] };
type BookInList = { Id: number; Cover: string; LastUpdatedAt: string; UserName: string; Title: string; SeriesTitle?: string | null };
type RemoteChapter = { Id: number; Title: string };
type RemoteBook = {
  Id: number; Cover: string; Title: string; Author?: string | null; Arthur?: string | null;
  Introduction?: string; LastUpdatedChapter?: string; LastUpdatedAt?: string;
  Extra?: { classification?: { tags?: string[] } };
  Chapter: RemoteChapter[];
};

export interface NovelSummary {
  source: string;
  id: string;
  title: string;
  cover_url: string | null;
  author: string | null;
  status: string | null;
  updated_at: string | null;
  description: string | null;
  tags: string[];
}
export interface NovelDetail extends NovelSummary {}
export interface ChapterSummary { id: string; title: string; }
export interface Volume { title: string; chapters: ChapterSummary[]; sections: Volume[]; }
export interface NovelOverview { detail: NovelDetail; volumes: Volume[]; readPosition: ServerReadPosition | null; }
export interface ServerReadPosition { chapterId: string; position: string; }
export interface ReaderDocument { id: string; bookId: string; chapterId: string; serverChapterId: string; title: string; html: string; readPosition: ServerReadPosition | null; }
export interface DiscoveryList { items: NovelSummary[]; pagination: { page: number; previous: number | null; next: number | null; first: number; last: number; }; }
export type RankingSort = "latest" | "view" | "new";
export interface RecommendBlock { title: string; items: NovelSummary[]; }

function summary(book: BookInList): NovelSummary {
  return { source: lightNovelSourceId, id: String(book.Id), title: book.Title, cover_url: book.Cover || null, author: book.UserName || null, status: book.SeriesTitle || null, updated_at: book.LastUpdatedAt || null, description: null, tags: [] };
}
function detail(book: RemoteBook): NovelDetail {
  return { source: lightNovelSourceId, id: String(book.Id), title: book.Title, cover_url: book.Cover || null, author: book.Author || book.Arthur || null, status: book.LastUpdatedChapter || null, updated_at: book.LastUpdatedAt || null, description: book.Introduction || null, tags: book.Extra?.classification?.tags ?? [] };
}
function list(raw: ListResult<BookInList>): DiscoveryList {
  return { items: raw.Data.map(summary), pagination: { page: raw.Page, previous: raw.Page > 1 ? raw.Page - 1 : null, next: raw.Page < raw.TotalPages ? raw.Page + 1 : null, first: 1, last: raw.TotalPages } };
}
function readPosition(value: unknown): ServerReadPosition | null {
  if (!value || typeof value !== "object") return null;
  const raw = value as { ChapterId?: number; Position?: string };
  return typeof raw.ChapterId === "number" ? { chapterId: String(raw.ChapterId), position: String(raw.Position ?? "") } : null;
}

export async function getLatest(page = 1): Promise<DiscoveryList> {
  return list(await requestWithHub<ListResult<BookInList>>("GetLatestBookList", { Page: page, Size: 24 }));
}
export async function getRanking(sort: RankingSort, page = 1): Promise<DiscoveryList> {
  return list(await requestWithHub<ListResult<BookInList>>("GetBookList", { Page: page, Size: 24, Order: sort }));
}
export async function searchDiscovery(query: string, page = 1): Promise<DiscoveryList> {
  return list(await requestWithHub<ListResult<BookInList>>("GetBookList", { Page: page, Size: 24, KeyWords: query }));
}
export async function getReaderOverview(_source: string, bookId: string): Promise<NovelOverview> {
  const response = await requestWithHub<{ Book: RemoteBook; ReadPosition?: unknown }>("GetBookInfo", { Id: Number(bookId) });
  const position = readPosition(response.ReadPosition);
  if (position) {
    const sortNum = response.Book.Chapter.findIndex((chapter) => String(chapter.Id) === position.chapterId) + 1;
    if (sortNum > 0) position.chapterId = String(sortNum);
  }
  return { detail: detail(response.Book), volumes: [{ title: "章节", chapters: response.Book.Chapter.map((chapter, index) => ({ id: String(index + 1), title: chapter.Title })), sections: [] }], readPosition: position };
}
export async function getReaderDocument(_source: string, bookId: string, documentId: string): Promise<ReaderDocument> {
  const response = await requestWithHub<{ Chapter: { Id: number; Title: string; Content: string }; ReadPosition?: unknown }>("GetNovelContent", { Bid: Number(bookId), SortNum: Number(documentId) });
  return { id: `${bookId}:${documentId}`, bookId, chapterId: documentId, serverChapterId: String(response.Chapter.Id), title: response.Chapter.Title, html: DOMPurify.sanitize(response.Chapter.Content ?? ""), readPosition: readPosition(response.ReadPosition) };
}
export async function saveReadPosition(bookId: string, chapterId: string, xpath: string): Promise<void> {
  await requestWithHub("SaveReadPosition", { Bid: Number(bookId), Cid: Number(chapterId), XPath: xpath });
}
export async function getBooksByIds(ids: number[]): Promise<NovelSummary[]> {
  if (!ids.length) return [];
  const batches = Array.from({ length: Math.ceil(ids.length / 24) }, (_, index) => ids.slice(index * 24, index * 24 + 24));
  return (await Promise.all(batches.map((Ids) => requestWithHub<BookInList[]>("GetBookListByIds", { Ids })))).flat().map(summary);
}
