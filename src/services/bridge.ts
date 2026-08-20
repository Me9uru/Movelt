import { invoke } from "@tauri-apps/api/core";

export async function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(name, args);
  } catch (error) {
    if (isAuthenticationExpired(error)) {
      window.dispatchEvent(new Event("movel:authentication-expired"));
    }
    throw error;
  }
}

function isAuthenticationExpired(error: unknown): boolean {
  if (error && typeof error === "object") {
    return (error as { code?: unknown }).code === "AUTHENTICATION_EXPIRED";
  }
  return typeof error === "string" && error.includes("登录已失效");
}
