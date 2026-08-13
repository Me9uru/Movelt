import { convertFileSrc, invoke, isTauri } from "@tauri-apps/api/core";
import type { NovelOverview } from "./novel";

export const localEpubSourceId = "local_epub";

export function canUseLocalEpubAssets(): boolean {
  return isTauri();
}

export function importEpub(path: string): Promise<NovelOverview> {
  return invoke("import_epub", { path });
}

export function getLocalEpubAssetDataUrl(bookId: string, resourcePath: string): Promise<string> {
  return invoke("get_local_epub_asset_data_url", { bookId, resourcePath });
}

export async function localEpubAssetUrl(bookId: string, path: string | null): Promise<string | null> {
  if (!path) return null;
  if (canUseLocalEpubAssets()) return convertFileSrc(path);
  return getLocalEpubAssetDataUrl(bookId, path);
}
