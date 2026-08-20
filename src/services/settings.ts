import { getCurrentWebview } from "@tauri-apps/api/webview";

export function clearWebviewCache(): Promise<void> {
  return getCurrentWebview().clearAllBrowsingData();
}
