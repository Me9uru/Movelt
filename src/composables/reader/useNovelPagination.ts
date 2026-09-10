import { nextTick, onBeforeUnmount, onMounted, ref, watch, type Ref } from "vue";
import type { NovelReaderSettings } from "../../types/reader";

export const useNovelPagination = (
  settings: NovelReaderSettings,
  chapterEntry: Ref<"default" | "next" | "previous">,
  restoreServerPosition: () => void,
) => {
  const pageViewport = ref<HTMLElement | null>(null);
  const currentPage = ref(0);
  const pageCount = ref(1);
  const pageLocation = ref(0);
  const hasRestoredPage = ref(false);
  const isSpread = ref(false);
  let resizeObserver: ResizeObserver | undefined;
  let spreadQuery: MediaQueryList | undefined;
  let paginationFrame: number | null = null;
  let paginationRequest = 0;
  let paginationResetPending = false;
  const updateSpread = (): void => { isSpread.value = Boolean(spreadQuery?.matches); };

  const pageStep = (): number => {
    const viewport = pageViewport.value;
    if (!viewport) return 0;
    const viewportStyle = getComputedStyle(viewport);
    const gap = Number.parseFloat(viewportStyle.columnGap) || 0;
    const padding =
      (Number.parseFloat(viewportStyle.paddingLeft) || 0) +
      (Number.parseFloat(viewportStyle.paddingRight) || 0);
    return Math.max(1, viewport.clientWidth - padding + gap);
  };

  const performPagination = (resetPage: boolean) => {
    const viewport = pageViewport.value;
    if (!viewport || settings.mode !== "paged") return;
    resizeObserver?.observe(viewport);

    const step = pageStep();
    const scrollDistance = Math.max(
      0,
      viewport.scrollWidth - viewport.clientWidth,
    );
    pageCount.value = Math.max(1, Math.floor(scrollDistance / step) + 1);
    if (resetPage && !hasRestoredPage.value) {
      pageLocation.value =
        chapterEntry.value !== "default"
          ? 0
          : pageLocation.value;
    }
    currentPage.value = Math.round(
      pageLocation.value * Math.max(0, pageCount.value - 1),
    );
    hasRestoredPage.value = true;
    viewport.scrollTo({ left: currentPage.value * step, behavior: "auto" });
    restoreServerPosition();
  };

  const updatePagination = (resetPage = false) => {
    if (settings.mode !== "paged") return;
    paginationResetPending ||= resetPage;
    const request = ++paginationRequest;
    void nextTick(() => {
      if (request !== paginationRequest || settings.mode !== "paged") return;
      if (paginationFrame !== null) window.cancelAnimationFrame(paginationFrame);
      paginationFrame = window.requestAnimationFrame(() => {
        paginationFrame = null;
        if (request !== paginationRequest) return;
        const shouldReset = paginationResetPending;
        paginationResetPending = false;
        performPagination(shouldReset);
      });
    });
  };

  const cancelPaginationUpdate = () => {
    paginationRequest++;
    paginationResetPending = false;
    if (paginationFrame !== null) {
      window.cancelAnimationFrame(paginationFrame);
      paginationFrame = null;
    }
  };


  watch(
    () => [
      settings.mode,
      settings.fontSize,
      settings.lineHeight,
      settings.letterSpacing,
      settings.paragraphSpacing,
      settings.contentWidth,
      settings.font,
      isSpread.value,
    ],
    () => updatePagination(true),
    { deep: true },
  );


  onMounted(() => {
    spreadQuery = window.matchMedia("(min-width: 960px)");
    updateSpread();
    spreadQuery.addEventListener("change", updateSpread);
    resizeObserver = new ResizeObserver(() => updatePagination());
    updatePagination(true);
  });
  watch(pageViewport, (viewport) => {
    resizeObserver?.disconnect();
    if (viewport) resizeObserver?.observe(viewport);
  });
  onBeforeUnmount(() => {
    cancelPaginationUpdate();
    spreadQuery?.removeEventListener("change", updateSpread);
    resizeObserver?.disconnect();
  });
  return { pageViewport, currentPage, pageCount, pageLocation, hasRestoredPage, isSpread, pageStep, updatePagination, cancelPaginationUpdate };
};
