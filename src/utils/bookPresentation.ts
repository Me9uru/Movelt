import type { NovelSummary } from "../domain/novel";
import type { ComicSummary } from "../domain/comic";
import type { BookGridItem } from "../types/book";

export const toNovelGridItem = (book: NovelSummary): BookGridItem<NovelSummary> => ({
  id: `${book.source}:${book.id}`,
  title: book.title,
  coverUrl: book.cover_url,
  data: book,
});

export const toComicGridItem = (comic: ComicSummary): BookGridItem<ComicSummary> => ({
  id: comic.id,
  title: comic.title,
  coverUrl: comic.coverUrl,
  data: comic,
});
