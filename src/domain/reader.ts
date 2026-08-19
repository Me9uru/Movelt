/** Sanitized LightNovelShelf chapter payload consumed by the reader. */
export interface ReaderDocument {
  id: string;
  bookId: string;
  chapterId: string;
  serverChapterId: string;
  title: string;
  html: string;
  readPosition: { chapterId: string; position: string } | null;
}
