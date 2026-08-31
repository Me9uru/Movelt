import { getCurrentWebview } from "@tauri-apps/api/webview";

export const clearWebviewCache = (): Promise<void> => {
  return getCurrentWebview().clearAllBrowsingData();
}
