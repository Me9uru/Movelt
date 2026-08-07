import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { ChapterContent, NovelOverview } from "./novel";

export const localEpubSourceId = "local_epub";

type TauriWindow = Window & {
  __TAURI_INTERNALS__?: {
    convertFileSrc?: (path: string, protocol?: string) => string;
  };
};

export function canUseLocalEpubAssets(): boolean {
  return typeof (window as TauriWindow).__TAURI_INTERNALS__?.convertFileSrc === "function";
}

export function importEpub(path: string): Promise<NovelOverview> {
  return invoke("import_epub", { path });
}

export function getLocalEpubOverview(bookId: string): Promise<NovelOverview> {
  return invoke("get_local_epub_overview", { bookId });
}

export function getLocalEpubChapter(bookId: string, chapterId: string): Promise<ChapterContent> {
  return invoke("get_local_epub_chapter", { bookId, chapterId });
}

export function getLocalEpubAssetDataUrl(bookId: string, resourcePath: string): Promise<string> {
  return invoke("get_local_epub_asset_data_url", { bookId, resourcePath });
}

export async function localEpubAssetUrl(bookId: string, path: string | null): Promise<string | null> {
  if (!path) return null;
  if (canUseLocalEpubAssets()) return convertFileSrc(path);
  return getLocalEpubAssetDataUrl(bookId, path);
}
