import { getReaderDocument } from "../services/novel";
import type { ReaderDocument, ReaderSource } from "../domain/reader";

/** 将后端已归一化的网络小说章节转换为阅读器文档。 */
export function networkNovelSource(sourceId: string): ReaderSource {
  return {
    id: sourceId,

    async loadDocument(bookId, documentId, documentTitle): Promise<ReaderDocument> {
      return getReaderDocument(sourceId, bookId, documentId, documentTitle);
    },
  };
}
