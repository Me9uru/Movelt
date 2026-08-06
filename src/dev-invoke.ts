interface TauriInternals {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

type WindowWithTauri = Window & {
  __TAURI_INTERNALS__?: TauriInternals;
};

/**
 * Makes @tauri-apps/api's normal invoke() function work in an external browser.
 * Vite proxies browser requests to the real Tauri invoke handler exposed on the
 * host's loopback interface in debug builds. Native Tauri webviews already have
 * __TAURI_INTERNALS__ and are left untouched.
 */
export function setupDevInvoke(
  endpoint = "/__tauri_invoke",
): void {
  const browserWindow = window as WindowWithTauri;
  if (browserWindow.__TAURI_INTERNALS__) return;

  browserWindow.__TAURI_INTERNALS__ = {
    async invoke<T>(
      command: string,
      args: Record<string, unknown> = {},
    ): Promise<T> {
      const response = await fetch(endpoint, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ cmd: command, args }),
      });

      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as {
          error?: string;
        } | null;
        throw new Error(body?.error ?? `Tauri invoke failed: HTTP ${response.status}`);
      }

      return response.json() as Promise<T>;
    },
  };
}
