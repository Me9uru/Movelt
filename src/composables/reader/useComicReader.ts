import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { saveComicReadPosition } from "../../services/comic";
import { useReaderSettings } from "./useReaderSettings";
import { useComicScroll } from "./useComicScroll";
import { useComicChapter } from "./useComicChapter";
import { useReaderLifecycle } from "./useReaderLifecycle";
import { showError } from "../../utils/error";
import { createReaderProgressSaver } from "../../utils/readerProgress";
import { pageOffsetForSide } from "../../utils/readerNavigation";

export const useComicReader = () => {
  const router = useRouter();
  const { settings } = useReaderSettings("comic");
  const settingsVisible = ref(false);
  let saveTimer: number | undefined;
  const chapter = useComicChapter(settings, () => {
    scroll.reset();
    if (saveTimer !== undefined) window.clearTimeout(saveTimer);
    saveTimer = undefined;
  }, (...args) => activateScrollReader(...args));
  const { comicId, chapterId, comic, pages, currentPage, savedPage, loadedComicId, loadedChapterId, loading, error, loadPage, load } = chapter;
  const scroll = useComicScroll(chapter, settings, (index) => setCurrentPage(index));
  const { pageList, activateScrollReader, handleImageLoad } = scroll;
  const hasPreviousChapter = computed(
    () =>
      (comic.value?.chapters.findIndex(
        (chapter) => chapter.id === chapterId.value,
      ) ?? 0) > 0,
  );
  const hasNextChapter = computed(() => {
    const chapters = comic.value?.chapters ?? [];
    const index = chapters.findIndex((chapter) => chapter.id === chapterId.value);
    return index >= 0 && index < chapters.length - 1;
  });

  const persistProgress = createReaderProgressSaver("comic", (value) => showError(value, "保存阅读进度失败"));
  const saveProgress = (immediate = false): void => {
    if (!comic.value || savedPage.value < 0 || !loadedComicId.value || !loadedChapterId.value)
      return;
    if (saveTimer !== undefined) window.clearTimeout(saveTimer);
    const save = () => {
      saveTimer = undefined;
      const currentComicId = loadedComicId.value;
      const currentServerChapterId = loadedChapterId.value;
      const page = savedPage.value + 1;
      persistProgress(
        currentComicId,
        () =>
          saveComicReadPosition(
            currentComicId,
            currentServerChapterId,
            page,
          ),
      );
    };
    if (immediate) save();
    else saveTimer = window.setTimeout(save, 400);
  };

  const setCurrentPage = (index: number): void => {
    if (index < 0 || index >= pages.value.length || index === currentPage.value)
      return;
    currentPage.value = index;
    savedPage.value = index;
    saveProgress();
  };

  const chapterOffset = (offset: number): void => {
    if (loading.value) return;
    const chapters = comic.value?.chapters ?? [];
    const index = chapters.findIndex((chapter) => chapter.id === chapterId.value);
    const next = chapters[index + offset];
    if (next) {
      savedPage.value = currentPage.value;
      saveProgress(true);
      void router.replace({
        name: "comic-reader",
        params: { comicId: comicId.value, chapterId: next.id },
      });
    }
  };

  /** 阅读页只持有分卷 ID；返回时复用来源页，避免将其误作系列标题。 */
  const returnToDetail = (): void => {
    if (window.history.state?.back) router.back();
    else void router.push({ name: "comic" });
  };

  const changePage = (offset: number): void => {
    if (loading.value) return;
    const next = currentPage.value + offset;
    if (next < 0 || next >= pages.value.length) {
      chapterOffset(offset);
      return;
    }
    setCurrentPage(next);
    void loadPage(next);
  };

  const handleReaderClick = (event: MouseEvent): void => {
    if ((event.target as HTMLElement).closest("button, a")) return;
    if (settings.mode === "paged") {
      const start = window.innerWidth / 3;
      const end = start * 2;
      if (event.clientX < start) {
        changePage(pageOffsetForSide(settings.pageTurnDirection, "left"));
        return;
      }
      if (event.clientX > end) {
        changePage(pageOffsetForSide(settings.pageTurnDirection, "right"));
        return;
      }
    }
    settingsVisible.value = true;
  };

  const saveCurrentProgress = (): void => {
    if (savedPage.value < 0 || !pages.value.length) return;
    savedPage.value = currentPage.value;
    saveProgress(true);
  };

  onMounted(() => {
    void load();
  });

  useReaderLifecycle(saveCurrentProgress);
  watch([comicId, chapterId], () => {
    if (!loading.value) saveCurrentProgress();
    void load();
  });

  watch(
    () => settings.mode,
    (mode) => {
      currentPage.value = Math.max(0, savedPage.value);
      scroll.reset();
      if (mode === "scroll")
        void activateScrollReader(currentPage.value);
      else void loadPage(currentPage.value);
    },
  );

  return {
    settings, comic, pages, loading, error, settingsVisible, pageList, currentPage,
    hasPreviousChapter, hasNextChapter, load, returnToDetail, handleReaderClick,
    chapterOffset, changePage, handleImageLoad,
  };
};
