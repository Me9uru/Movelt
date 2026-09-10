import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useReaderSettings } from "./useReaderSettings";
import { useNovelChapter } from "./useNovelChapter";
import { useNovelPagination } from "./useNovelPagination";
import { useNovelReadingPosition } from "./useNovelReadingPosition";
import { useReaderLifecycle } from "./useReaderLifecycle";
import { pageOffsetForSide } from "../../utils/readerNavigation";

export const useNovelReader = () => {
  const { settings, style } = useReaderSettings("novel");
  const chapter = useNovelChapter(settings, () => recordProgress());
  const { bookId, readerDocument, loading, error, chapterEntry, hasPreviousChapter, hasNextChapter, load, changeChapter } = chapter;
  const readerRoot = ref<HTMLElement | null>(null);
  const readerContent = ref<HTMLElement | null>(null);
  const previewVisible = ref(false);
  const settingsVisible = ref(false);
  const pagination = useNovelPagination(settings, chapterEntry, () => restoreServerPosition());
  const { pageViewport, currentPage, pageCount, pageLocation, hasRestoredPage, isSpread, pageStep, updatePagination, cancelPaginationUpdate } = pagination;
  const { recordProgress, restoreServerPosition, hasRestoredServerPosition } = useNovelReadingPosition({
    settings, ...chapter, readerContent, ...pagination,
  });
  let pointerStartX: number | null = null;
  let suppressReaderClickUntil = 0;
  let scrollTimer: number | null = null;
  let hasRestoredScroll = false;
  let nextChapterRequested = false;
  let previousChapterRequested = false;
  const pageLabel = computed(() => `${currentPage.value + 1} / ${pageCount.value}`);
  const clampLocation = (location: number): number => Math.min(1, Math.max(0, location));
  const setReaderContent = (element: HTMLElement): void => {
    readerContent.value = element;
    if (settings.mode === "scroll") void nextTick(restoreServerPosition);
  };
  const cancelPointer = (): void => { pointerStartX = null; };

  const goToPage = (page: number) => {
    const viewport = pageViewport.value;
    if (!viewport) return;
    if (page >= pageCount.value) {
      recordProgress();
      requestNextChapter();
      return;
    }
    if (page < 0) {
      recordProgress();
      requestPreviousChapter();
      return;
    }
    if (page < pageCount.value - 1) nextChapterRequested = false;
    currentPage.value = Math.min(Math.max(page, 0), pageCount.value - 1);
    if (pageCount.value > 1) {
      pageLocation.value = currentPage.value / (pageCount.value - 1);
    }
    viewport.scrollTo({
      left: currentPage.value * pageStep(),
      behavior: "smooth",
    });
    recordProgress();
  };

  const scrollMetrics = (): { start: number; distance: number; } | null => {
    const reader = readerRoot.value;
    if (!reader) return null;
    const start = reader.getBoundingClientRect().top + window.scrollY;
    return {
      start,
      distance: Math.max(1, reader.scrollHeight - window.innerHeight),
    };
  };

  const recordScrollProgress = (): number | null => {
    const metrics = scrollMetrics();
    if (!metrics) return null;
    pageLocation.value = clampLocation(
      (window.scrollY - metrics.start) / metrics.distance,
    );
    recordProgress();
    return pageLocation.value;
  };

  const requestNextChapter = () => {
    if (nextChapterRequested || loading.value || !hasNextChapter.value) return;
    nextChapterRequested = true;
    changeChapter(1);
  };

  const requestPreviousChapter = () => {
    if (previousChapterRequested || loading.value || !hasPreviousChapter.value)
      return;
    previousChapterRequested = true;
    changeChapter(-1);
  };

  const handleScroll = () => {
    if (settings.mode !== "scroll") return;
    if (scrollTimer !== null) window.clearTimeout(scrollTimer);
    scrollTimer = window.setTimeout(() => {
      scrollTimer = null;
      if (recordScrollProgress() === null) return;
    }, 120);
  };

  const handlePageScroll = (): void => {
    if (settings.mode !== "paged") return;
    if (scrollTimer !== null) window.clearTimeout(scrollTimer);
    scrollTimer = window.setTimeout(() => {
      scrollTimer = null;
      recordProgress();
    }, 120);
  };

  const restoreScrollProgress = () => {
    if (settings.mode !== "scroll" || hasRestoredScroll) return;
    void nextTick(() => {
      const metrics = scrollMetrics();
      if (!metrics) return;
      hasRestoredScroll = true;
      window.scrollTo({
        top: metrics.start + pageLocation.value * metrics.distance,
        behavior: "auto",
      });
    });
  };

  const handleKeydown = (event: KeyboardEvent) => {
    if (previewVisible.value || settingsVisible.value) return;
    if (settings.mode !== "paged") return;
    const target = event.target;
    if (
      target instanceof HTMLElement &&
      target.closest("input, button, [role='button'], [role='slider'], [role='radio'], [role='dialog'], [contenteditable='true']")
    )
      return;
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      goToPage(currentPage.value + pageOffsetForSide(settings.pageTurnDirection, "left"));
    } else if (event.key === "ArrowRight" || event.key === " ") {
      event.preventDefault();
      goToPage(currentPage.value + (event.key === " " ? 1 : pageOffsetForSide(settings.pageTurnDirection, "right")));
    }
  };

  const handlePointerDown = (event: PointerEvent) => {
    if (previewVisible.value) return;
    pointerStartX = event.clientX;
  };

  const handlePointerUp = (event: PointerEvent) => {
    if (previewVisible.value) {
      pointerStartX = null;
      return;
    }
    if (pointerStartX === null) return;
    const distance = event.clientX - pointerStartX;
    pointerStartX = null;
    if (Math.abs(distance) < 45) return;
    suppressReaderClickUntil = performance.now() + 350;
    goToPage(currentPage.value + (distance < 0 ? 1 : -1));
  };

  const handleReaderClick = (event: MouseEvent) => {
    if (previewVisible.value) return;
    if (performance.now() < suppressReaderClickUntil) return;

    const target = event.target;
    if (
      target instanceof HTMLElement &&
      target.closest(
        "button, a, input, label, [role='button'], [contenteditable='true'], .var-slider, .var-radio-group",
      )
    )
      return;
    if (window.getSelection()?.isCollapsed === false) return;

    const middleStart = window.innerWidth / 3;
    const middleEnd = middleStart * 2;
    if (event.clientX < middleStart && settings.mode === "paged") {
      goToPage(currentPage.value + pageOffsetForSide(settings.pageTurnDirection, "left"));
    } else if (event.clientX > middleEnd && settings.mode === "paged") {
      goToPage(currentPage.value + pageOffsetForSide(settings.pageTurnDirection, "right"));
    } else if (event.clientX >= middleStart && event.clientX <= middleEnd) {
      settingsVisible.value = true;
    }
  };

  watch(
    () => readerDocument.value,
    () => {
      readerContent.value = null;
      nextChapterRequested = false;
      previousChapterRequested = false;
      hasRestoredPage.value = false;
      hasRestoredScroll = false;
      hasRestoredServerPosition.value = chapterEntry.value !== "default";
      pageLocation.value = 0;
      void nextTick(() => {
        updatePagination(true);
        if (settings.mode === "scroll") restoreServerPosition();
      });
    },
    { immediate: true },
  );

  watch(
    () => settings.mode,
    (mode) => {
      if (mode === "scroll") {
        cancelPaginationUpdate();
        hasRestoredScroll = false;
        restoreScrollProgress();
      } else {
        hasRestoredPage.value = false;
      }
    },
  );

  onMounted(() => {
    window.addEventListener("keydown", handleKeydown);
    window.addEventListener("scroll", handleScroll, { passive: true });
    restoreScrollProgress();
  });
  useReaderLifecycle(recordProgress);

  onBeforeUnmount(() => {
    window.removeEventListener("keydown", handleKeydown);
    window.removeEventListener("scroll", handleScroll);
    if (scrollTimer !== null) window.clearTimeout(scrollTimer);
  });

  return {
    settings, style, bookId, readerDocument, loading, error, load, changeChapter,
    readerRoot, pageViewport, previewVisible, settingsVisible, currentPage, pageCount,
    isSpread, pageLabel, hasPreviousChapter, hasNextChapter, setReaderContent,
    updatePagination, handleReaderClick, handlePointerDown, handlePointerUp, cancelPointer, handlePageScroll,
    requestPreviousChapter, requestNextChapter, goToPage,
  };
};
