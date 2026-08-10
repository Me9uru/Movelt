import { getChapter, prefetchChapters } from "../services/novel";
import type { ReaderDocument, ReaderSource } from "../domain/reader";

/** 将后端已归一化的网络小说章节转换为阅读器文档。 */
export function networkNovelSource(sourceId: string): ReaderSource {
  return {
    id: sourceId,

    async loadDocument(bookId, documentId, documentTitle): Promise<ReaderDocument> {
      const chapter = await getChapter(sourceId, bookId, documentId, documentTitle);
      return {
        id: `${chapter.source}:${chapter.novel_id}:${chapter.chapter_id}`,
        sourceId: chapter.source,
        bookId: chapter.novel_id,
        title: chapter.title,
        blocks: chapter.nodes,
      };
    },

    async prefetchDocuments(bookId, documentIds, documentTitles): Promise<void> {
      await prefetchChapters(sourceId, bookId, documentIds, documentTitles);
    },
  };
}
