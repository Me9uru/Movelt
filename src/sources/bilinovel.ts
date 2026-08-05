import { getChapter } from "../services/novel";
import type { ReaderDocument, ReaderSource } from "../domain/reader";

/** Bilinovel 的传输格式只在这里出现，阅读器不依赖在线接口字段。 */
export const bilinovelSource: ReaderSource = {
  id: "bilinovel",

  async loadDocument(bookId, documentId): Promise<ReaderDocument> {
    const chapter = await getChapter(bookId, documentId);
    return {
      id: `${chapter.source}:${chapter.novel_id}:${chapter.chapter_id}`,
      sourceId: chapter.source,
      bookId: chapter.novel_id,
      title: chapter.title,
      blocks: chapter.nodes,
    };
  },
};
