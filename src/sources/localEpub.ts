import type { ReaderDocument, ReaderSource } from "../domain/reader";
import { getLocalEpubChapter, localEpubAssetUrl, localEpubSourceId } from "../services/localEpub";

export const localEpubSource: ReaderSource = {
  id: localEpubSourceId,
  async loadDocument(bookId, documentId): Promise<ReaderDocument> {
    const chapter = await getLocalEpubChapter(bookId, documentId);
    const blocks = await Promise.all(chapter.nodes.map(async (node) => {
      if (node.type !== "image") return node;
      return { ...node, url: await localEpubAssetUrl(bookId, node.url) ?? node.url };
    }));
    return {
      id: `${chapter.source}:${chapter.novel_id}:${chapter.chapter_id}`,
      sourceId: chapter.source,
      bookId: chapter.novel_id,
      title: chapter.title,
      blocks,
    };
  },
};
