import { Dialog, Snackbar } from "@varlet/ui";

const fallbackMessage = "操作失败，请稍后重试";
let activeDialogMessage: string | null = null;

export const getErrorMessage = (
  error: unknown,
  fallback = fallbackMessage,
): string => {
  if (typeof error === "string" && error.trim()) return error;
  if (error && typeof error === "object") {
    const value = error as { message?: unknown; code?: unknown };
    if (typeof value.message === "string" && value.message.trim())
      return value.message;
    if (typeof value.code === "string" && value.code.trim()) return value.code;
  }
  return fallback;
}

/** Keeps non-page-specific failures visible to developers without interrupting the UI. */
export const showError = (error: unknown, fallback?: string): void => {
  const message = getErrorMessage(error, fallback);
  console.error(message);
  Snackbar.error({ content: message, lockScroll: false });
}

/** Shows fatal navigation and rendering errors in a centered, dismissible dialog. */
export const showErrorDialog = (error: unknown, fallback?: string): void => {
  const message = getErrorMessage(error, fallback);
  console.error(message);
  if (activeDialogMessage === message) return;

  activeDialogMessage = message;
  void Dialog({
    title: "操作失败",
    message,
    dialogClass: "error-dialog",
    confirmButtonText: "知道了",
    closeOnClickOverlay: false,
  }).finally(() => {
    activeDialogMessage = null;
  });
}
