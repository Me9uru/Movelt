import type { ReaderDocument, ReaderSource } from "../domain/reader";
import { getReaderDocument } from "../services/novel";
import { localEpubAssetUrl, localEpubSourceId } from "../services/localEpub";

export const localEpubSource: ReaderSource = {
  id: localEpubSourceId,
  async loadDocument(bookId, documentId): Promise<ReaderDocument> {
    const document = await getReaderDocument(localEpubSourceId, bookId, documentId);
    const blocks = await Promise.all(document.blocks.map(async (node) => {
      if (node.type !== "image") return node;
      return { ...node, url: await localEpubAssetUrl(bookId, node.url) ?? node.url };
    }));
    return {
      ...document,
      blocks,
    };
  },
};
