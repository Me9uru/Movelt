export type ReaderProgressKind = "novel" | "comic";

export interface ReaderProgressCheckpoint {
  itemId: string;
  routeChapterId: string;
  serverChapterId: string;
  position: string;
  updatedAt: number;
}

interface PendingSave {
  checkpoint: ReaderProgressCheckpoint;
  token: string;
  persist: () => Promise<void>;
}

const storagePrefix = "movel:reader-progress";

const storageKey = (kind: ReaderProgressKind, itemId: string): string => {
  return `${storagePrefix}:${kind}:${itemId}`;
}

export const readReaderProgress = (
  kind: ReaderProgressKind,
  itemId: string,
): ReaderProgressCheckpoint | null => {
  try {
    const value = JSON.parse(
      localStorage.getItem(storageKey(kind, itemId)) ?? "null",
    ) as Partial<ReaderProgressCheckpoint> | null;
    if (
      !value ||
      value.itemId !== itemId ||
      typeof value.routeChapterId !== "string" ||
      typeof value.serverChapterId !== "string" ||
      typeof value.position !== "string" ||
      typeof value.updatedAt !== "number"
    )
      return null;
    return value as ReaderProgressCheckpoint;
  } catch {
    return null;
  }
}

const writeReaderProgress = (
  kind: ReaderProgressKind,
  checkpoint: ReaderProgressCheckpoint,
): string => {
  const token = JSON.stringify(checkpoint);
  try {
    localStorage.setItem(storageKey(kind, checkpoint.itemId), token);
  } catch {
    // Cloud persistence still works when WebView storage is unavailable.
  }
  return token;
}

const clearReaderProgress = (
  kind: ReaderProgressKind,
  checkpoint: ReaderProgressCheckpoint,
  token: string,
): void => {
  try {
    const key = storageKey(kind, checkpoint.itemId);
    if (localStorage.getItem(key) === token) localStorage.removeItem(key);
  } catch {
    // Storage cleanup is best-effort.
  }
}

/**
 * Writes a synchronous recovery checkpoint before starting the remote save.
 * Remote writes are serialized and coalesced so an older request cannot arrive
 * after a newer reading position and move the cloud position backwards.
 */
export const createReaderProgressSaver = (
  kind: ReaderProgressKind,
  onError: (error: unknown) => void,
) => {
  let pending: PendingSave | null = null;
  let saving = false;

  const drain = async (): Promise<void> => {
    if (saving) return;
    saving = true;
    while (pending) {
      const current = pending;
      pending = null;
      try {
        await current.persist();
        clearReaderProgress(kind, current.checkpoint, current.token);
      } catch (error) {
        onError(error);
      }
    }
    saving = false;
  }

  return (checkpoint: Omit<ReaderProgressCheckpoint, "updatedAt">, persist: () => Promise<void>) => {
    const savedCheckpoint = { ...checkpoint, updatedAt: Date.now() };
    pending = {
      checkpoint: savedCheckpoint,
      token: writeReaderProgress(kind, savedCheckpoint),
      persist,
    };
    void drain();
  };
}
