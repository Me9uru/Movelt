import { ElMessage } from "element-plus";

const fallbackMessage = "操作失败，请稍后重试";

export function getErrorMessage(error: unknown, fallback = fallbackMessage): string {
  if (typeof error === "string" && error.trim()) return error;
  if (error && typeof error === "object") {
    const value = error as { message?: unknown; code?: unknown };
    if (typeof value.message === "string" && value.message.trim()) return value.message;
    if (typeof value.code === "string" && value.code.trim()) return value.code;
  }
  return fallback;
}

/** Keeps non-page-specific failures visible to developers without interrupting the UI. */
export function showError(error: unknown, fallback?: string): void {
  const message = getErrorMessage(error, fallback);
  console.error(message);
  ElMessage.error({ message, grouping: true });
}
