export type ReaderBlock =
  | { type: "paragraph"; text: string }
  | { type: "image"; url: string; alt: string | null };

/**
 * 阅读器唯一认识的数据结构。
 * 网络章节、EPUB 和 TXT 都应在各自的数据源适配器中转换成它。
 */
export interface ReaderDocument {
  id: string;
  sourceId: string;
  bookId: string;
  title: string;
  blocks: ReaderBlock[];
}

export interface ReaderSource {
  readonly id: string;
  loadDocument(bookId: string, documentId: string): Promise<ReaderDocument>;
  prefetchDocuments?(bookId: string, documentIds: string[]): Promise<void>;
}
