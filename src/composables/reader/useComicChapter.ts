import { computed, onBeforeUnmount, ref } from "vue";
import { useRoute } from "vue-router";
import type { ComicBook } from "../../domain/comic";
import type { ComicReaderSettings } from "../../types/reader";
import { getComicBook, getComicChapterPages } from "../../services/comic";
import { getErrorMessage, showError } from "../../utils/error";

export const useComicChapter = (
  settings: ComicReaderSettings,
  onBeforeLoad: () => void,
  activateScrollReader: (index: number, request: number, comicId: string, chapterId: string) => Promise<void>,
) => {
  const route = useRoute();
  const comicId = computed(() => typeof route.params.comicId === "string" ? route.params.comicId : "");
  const chapterId = computed(() => typeof route.params.chapterId === "string" ? route.params.chapterId : "");
  const comic = ref<ComicBook | null>(null);
  const pages = ref<string[]>([]);
  const currentPage = ref(0);
  const savedPage = ref(-1);
  const loadedComicId = ref("");
  const loadedChapterId = ref("");
  const loading = ref(true);
  const error = ref("");
  const loadRequest = ref(0);
  const requestedBatches = new Set<string>();
  const isActiveLoad = (
    request: number,
    requestComicId: string,
    requestChapterId: string,
  ): boolean => {
    return (
      request === loadRequest.value &&
      requestComicId === comicId.value &&
      requestChapterId === chapterId.value
    );
  };

  const loadPage = async (
    index: number,
    request = loadRequest.value,
  ): Promise<void> => {
    if (index < 0 || index >= pages.value.length || pages.value[index]) return;
    const batchStart = Math.floor(index / 12) * 12;
    const batchKey = `${request}:${batchStart}`;
    if (requestedBatches.has(batchKey)) return;
    requestedBatches.add(batchKey);
    const requestComicId = comicId.value;
    const requestChapterId = chapterId.value;
    try {
      const batch = await getComicChapterPages(
        requestComicId,
        requestChapterId,
        index,
      );
      if (!isActiveLoad(request, requestComicId, requestChapterId)) return;
      batch.pageUrls.forEach((url, offset) => {
        if (batch.startIndex + offset < pages.value.length)
          pages.value[batch.startIndex + offset] = url;
      });
    } catch (value) {
      if (!isActiveLoad(request, requestComicId, requestChapterId)) return;
      error.value = getErrorMessage(value, "无法加载漫画页面");
      showError(value, "无法加载漫画页面");
    } finally {
      requestedBatches.delete(batchKey);
    }
  };

  const load = async (): Promise<void> => {
    const request = ++loadRequest.value;
    const requestComicId = comicId.value;
    const requestChapterId = chapterId.value;
    loading.value = true;
    error.value = "";
    onBeforeLoad();
    requestedBatches.clear();
    savedPage.value = -1;
    currentPage.value = 0;
    pages.value = [];
    const pagesRequest = getComicChapterPages(
      requestComicId,
      requestChapterId,
      0,
    );
    const detailRequest = getComicBook(requestComicId);
    try {
      const [list, detail] = await Promise.all([pagesRequest, detailRequest]);
      if (!isActiveLoad(request, requestComicId, requestChapterId)) return;
      comic.value = detail;
      loadedComicId.value = detail.id;
      loadedChapterId.value = list.chapterId;
      pages.value = Array.from(
        { length: list.pageCount },
        (_, index) => list.pageUrls[index] || "",
      );
      const position = list.readPosition ?? detail.readPosition;
      const page =
        position?.chapterId === requestChapterId
          ? Number.parseInt(position.position, 10)
          : 1;
      savedPage.value = Math.min(
        Math.max(Number.isFinite(page) ? page - 1 : 0, 0),
        Math.max(0, pages.value.length - 1),
      );
      currentPage.value = savedPage.value;
      await loadPage(currentPage.value, request);
      if (!isActiveLoad(request, requestComicId, requestChapterId)) return;
      if (settings.mode === "scroll")
        await activateScrollReader(
          currentPage.value,
          request,
          requestComicId,
          requestChapterId,
        );
    } catch (value) {
      if (!isActiveLoad(request, requestComicId, requestChapterId)) return;
      error.value = getErrorMessage(value, "无法加载漫画页面");
      showError(value, "无法加载漫画页面");
    } finally {
      if (isActiveLoad(request, requestComicId, requestChapterId))
        loading.value = false;
    }
  };


  onBeforeUnmount(() => { loadRequest.value += 1; });
  return { comicId, chapterId, comic, pages, currentPage, savedPage, loadedComicId, loadedChapterId, loading, error, loadRequest, isActiveLoad, loadPage, load };
};
