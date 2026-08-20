interface TauriInternals {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

type WindowWithTauri = Window & {
  __TAURI_INTERNALS__?: TauriInternals;
};

/**
 * Routes browser debugging invokes through the debug-only localhost bridge.
 * Native Tauri webviews already provide this interface and are left unchanged.
 */
export function setupDevInvoke(endpoint = "/__tauri_invoke"): void {
  const browserWindow = window as WindowWithTauri;
  if (browserWindow.__TAURI_INTERNALS__) return;

  browserWindow.__TAURI_INTERNALS__ = {
    async invoke<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
      const response = await fetch(endpoint, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ cmd: command, args }),
      });

      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as {
          error?: { code?: string; message?: string } | string;
        } | null;
        const error = body?.error;
        if (error && typeof error === "object") throw error;
        throw new Error(error ?? `Tauri invoke failed: HTTP ${response.status}`);
      }

      return response.json() as Promise<T>;
    },
  };
}
