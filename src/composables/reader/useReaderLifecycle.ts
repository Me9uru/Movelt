import { onBeforeUnmount, onMounted } from "vue";

/** Keep the last visible location when the reader is hidden or removed. */
export const useReaderLifecycle = (save: () => void): void => {
  const onVisibilityChange = (): void => {
    if (document.visibilityState === "hidden") save();
  };
  onMounted(() => {
    document.addEventListener("visibilitychange", onVisibilityChange);
    window.addEventListener("pagehide", save);
  });
  onBeforeUnmount(() => {
    save();
    document.removeEventListener("visibilitychange", onVisibilityChange);
    window.removeEventListener("pagehide", save);
  });
};
