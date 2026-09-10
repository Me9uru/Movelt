import { nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import type { ComicReaderSettings } from "../../types/reader";
import type { useComicChapter } from "./useComicChapter";

export const useComicScroll = (
  chapter: ReturnType<typeof useComicChapter>,
  settings: ComicReaderSettings,
  setCurrentPage: (index: number) => void,
) => {
  const { pages, comicId, chapterId, loadRequest, isActiveLoad, loadPage } = chapter;
  const pageList = ref<HTMLElement | null>(null);
  let loadObserver: IntersectionObserver | undefined;
  let scrollFrame: number | undefined;
  let progressTrackingReady = false;
  let activation: AbortController | undefined;
  const reset = (): void => {
    activation?.abort();
    progressTrackingReady = false;
    loadObserver?.disconnect();
    if (scrollFrame !== undefined) cancelAnimationFrame(scrollFrame);
    scrollFrame = undefined;
  };
  const nextFrame = (): Promise<void> => {
    return new Promise((resolve) => requestAnimationFrame(() => resolve()));
  };

  const waitForImage = async (image: HTMLImageElement, signal: AbortSignal): Promise<void> => {
    if (image.complete || signal.aborted) return;
    await new Promise<void>((resolve) => {
      const finish = (): void => {
        image.removeEventListener("load", finish);
        image.removeEventListener("error", finish);
        signal.removeEventListener("abort", finish);
        resolve();
      };
      image.addEventListener("load", finish, { once: true });
      image.addEventListener("error", finish, { once: true });
      signal.addEventListener("abort", finish, { once: true });
    });
  };

  const currentPageFromLayout = (): number | null => {
    const list = pageList.value;
    if (!list) return null;
    const marker = Math.max(0, list.getBoundingClientRect().top) + 1;
    const pageElements = list.querySelectorAll<HTMLElement>(
      ".comic-reader-page",
    );
    for (const page of pageElements) {
      if (page.getBoundingClientRect().bottom <= marker) continue;
      const index = Number(page.dataset.pageIndex);
      return Number.isInteger(index) ? index : null;
    }
    return pages.value.length ? pages.value.length - 1 : null;
  };

  const updateCurrentPageFromLayout = (): void => {
    if (!progressTrackingReady || settings.mode !== "scroll") return;
    const index = currentPageFromLayout();
    if (index !== null) setCurrentPage(index);
  };

  const handleScroll = (): void => {
    if (!progressTrackingReady || settings.mode !== "scroll") return;
    if (scrollFrame !== undefined) cancelAnimationFrame(scrollFrame);
    scrollFrame = requestAnimationFrame(() => {
      scrollFrame = undefined;
      updateCurrentPageFromLayout();
    });
  };

  const activateScrollReader = async (
    index: number,
    request = loadRequest.value,
    requestComicId = comicId.value,
    requestChapterId = chapterId.value,
  ): Promise<void> => {
    reset();
    const controller = new AbortController();
    activation = controller;
    await nextTick();
    const target = pageList.value?.querySelector<HTMLElement>(
      `[data-page-index="${index}"]`,
    );
    const image = target?.querySelector<HTMLImageElement>("img");
    if (image) await waitForImage(image, controller.signal);
    await nextTick();
    if (
      controller.signal.aborted || !isActiveLoad(request, requestComicId, requestChapterId) ||
      settings.mode !== "scroll"
    )
      return;
    target?.scrollIntoView({ block: "start", behavior: "auto" });
    await nextFrame();
    if (
      controller.signal.aborted || !isActiveLoad(request, requestComicId, requestChapterId) ||
      settings.mode !== "scroll"
    )
      return;
    observePages();
    progressTrackingReady = true;
    updateCurrentPageFromLayout();
  };

  const observePages = (): void => {
    loadObserver?.disconnect();
    if (settings.mode !== "scroll") return;
    loadObserver = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          const index = Number((entry.target as HTMLElement).dataset.pageIndex);
          if (Number.isInteger(index)) void loadPage(index);
        }
      },
      { rootMargin: "800px 0px", threshold: 0.01 },
    );
    pageList.value
      ?.querySelectorAll<HTMLElement>(".comic-reader-page")
      .forEach((page) => {
        loadObserver?.observe(page);
      });
  };

  const handleImageLoad = (): void => { handleScroll(); };


  onMounted(() => window.addEventListener("scroll", handleScroll, { passive: true }));
  onBeforeUnmount(() => {
    reset();
    window.removeEventListener("scroll", handleScroll);
  });
  return { pageList, activateScrollReader, handleImageLoad, reset };
};
