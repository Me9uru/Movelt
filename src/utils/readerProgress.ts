import type { ReaderKind } from "../types/reader";

interface PendingSave {
  persist: () => Promise<void>;
  onError: (error: unknown) => void;
}

interface SaveQueue {
  pending: PendingSave | null;
  saving: boolean;
}

const saveQueues = new Map<string, SaveQueue>();
const obsoleteStoragePrefix = "movel:reader-progress:";

// Remove checkpoints written by versions that restored progress locally.
try {
  for (let index = localStorage.length - 1; index >= 0; index -= 1) {
    const key = localStorage.key(index);
    if (key?.startsWith(obsoleteStoragePrefix)) localStorage.removeItem(key);
  }
} catch {
  // Progress saving still works when WebView storage is unavailable.
}

const queueKey = (kind: ReaderKind, itemId: string): string => {
  return `${kind}:${itemId}`;
};

/**
 * Serializes and coalesces remote progress writes for one item. Queues live at
 * module scope so a save started by an unmounted reader still finishes before
 * a newly mounted reader writes a newer position for the same item.
 */
export const createReaderProgressSaver = (
  kind: ReaderKind,
  onError: (error: unknown) => void,
) => {
  return (itemId: string, persist: () => Promise<void>): void => {
    const key = queueKey(kind, itemId);
    const queue = saveQueues.get(key) ?? { pending: null, saving: false };
    queue.pending = { persist, onError };
    saveQueues.set(key, queue);

    if (queue.saving) return;
    queue.saving = true;
    void (async () => {
      while (queue.pending) {
        const current = queue.pending;
        queue.pending = null;
        try {
          await current.persist();
        } catch (error) {
          current.onError(error);
        }
      }
      queue.saving = false;
      if (!queue.pending) saveQueues.delete(key);
    })();
  };
};
