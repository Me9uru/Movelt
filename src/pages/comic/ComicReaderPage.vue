<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  getComicChapterPages,
  getComicBook,
  saveComicReadPosition,
} from "../../services/comic";
import type { ComicBook } from "../../domain/comic";
import { useReaderSettings } from "../../composables/reader/useReaderSettings";
import { getErrorMessage, showError } from "../../utils/error";
import ErrorState from "../../components/common/ErrorState.vue";
import ReaderToolbar from "../../components/reader/ReaderToolbar.vue";
import ReaderSettingsDrawer from "../../components/reader/ReaderSettingsDrawer.vue";
import ReaderBoundarySwitch from "../../components/reader/ReaderBoundarySwitch.vue";
import LoadingOverlay from "../../components/common/LoadingOverlay.vue";
import { createReaderProgressSaver } from "../../utils/readerProgress";

const route = useRoute();
const router = useRouter();
const comicId = computed(() => String(route.params.comicId));
const chapterId = computed(() => String(route.params.chapterId));
const { settings } = useReaderSettings("comic");
const comic = ref<ComicBook | null>(null);
const pages = ref<string[]>([]);
const loading = ref(true);
const error = ref("");
const settingsVisible = ref(false);
const pageList = ref<HTMLElement | null>(null);
const currentPage = ref(0);
const loadedPages = ref<boolean[]>([]);
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
const requestedBatches = new Set<string>();
let loadObserver: IntersectionObserver | undefined;
let savedPage = -1;
let saveTimer: number | undefined;
let loadRequest = 0;
let scrollFrame: number | undefined;
let progressTrackingReady = false;
// Vue Router 会在组件卸载前更新 route；保存进度必须使用本次已加载的
// 官方数字分卷 ID，而不能在卸载钩子中重新读取路由参数。
let loadedComicId = "";
let loadedChapterId = "";
const persistProgress = createReaderProgressSaver("comic", (value) =>
  showError(value, "保存阅读进度失败"),
);

const isActiveLoad = (
  request: number,
  requestComicId: string,
  requestChapterId: string,
): boolean => {
  return (
    request === loadRequest &&
    requestComicId === comicId.value &&
    requestChapterId === chapterId.value
  );
}

const loadPage = async (
  index: number,
  request = loadRequest,
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
}

const nextFrame = (): Promise<void> => {
  return new Promise((resolve) => requestAnimationFrame(() => resolve()));
}

const waitForImage = async (image: HTMLImageElement): Promise<void> => {
  if (image.complete) return;
  await new Promise<void>((resolve) => {
    image.addEventListener("load", () => resolve(), { once: true });
    image.addEventListener("error", () => resolve(), { once: true });
  });
}

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
}

const updateCurrentPageFromLayout = (): void => {
  if (!progressTrackingReady || settings.mode !== "scroll") return;
  const index = currentPageFromLayout();
  if (index !== null) setCurrentPage(index);
}

const handleScroll = (): void => {
  if (!progressTrackingReady || settings.mode !== "scroll") return;
  if (scrollFrame !== undefined) cancelAnimationFrame(scrollFrame);
  scrollFrame = requestAnimationFrame(() => {
    scrollFrame = undefined;
    updateCurrentPageFromLayout();
  });
}

const activateScrollReader = async (
  index: number,
  request = loadRequest,
  requestComicId = comicId.value,
  requestChapterId = chapterId.value,
): Promise<void> => {
  progressTrackingReady = false;
  await nextTick();
  const target = pageList.value?.querySelector<HTMLElement>(
    `[data-page-index="${index}"]`,
  );
  const image = target?.querySelector<HTMLImageElement>("img");
  if (image) await waitForImage(image);
  await nextTick();
  if (
    !isActiveLoad(request, requestComicId, requestChapterId) ||
    settings.mode !== "scroll"
  )
    return;
  target?.scrollIntoView({ block: "start", behavior: "auto" });
  await nextFrame();
  if (
    !isActiveLoad(request, requestComicId, requestChapterId) ||
    settings.mode !== "scroll"
  )
    return;
  observePages();
  progressTrackingReady = true;
  updateCurrentPageFromLayout();
}

const load = async (): Promise<void> => {
  const request = ++loadRequest;
  const requestComicId = comicId.value;
  const requestChapterId = chapterId.value;
  loading.value = true;
  error.value = "";
  progressTrackingReady = false;
  loadObserver?.disconnect();
  requestedBatches.clear();
  if (saveTimer !== undefined) window.clearTimeout(saveTimer);
  saveTimer = undefined;
  savedPage = -1;
  currentPage.value = 0;
  pages.value = [];
  loadedPages.value = [];
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
    loadedComicId = detail.id;
    loadedChapterId = list.chapterId;
    pages.value = Array.from(
      { length: list.pageCount },
      (_, index) => list.pageUrls[index] || "",
    );
    loadedPages.value = Array.from({ length: list.pageCount }, () => false);
    const position = list.readPosition ?? detail.readPosition;
    const page =
      position?.chapterId === requestChapterId
        ? Number.parseInt(position.position, 10)
        : 1;
    savedPage = Math.min(
      Math.max(Number.isFinite(page) ? page - 1 : 0, 0),
      Math.max(0, pages.value.length - 1),
    );
    currentPage.value = savedPage;
    await nextTick();
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
}

const saveProgress = (immediate = false): void => {
  if (!comic.value || savedPage < 0 || !loadedComicId || !loadedChapterId)
    return;
  if (saveTimer !== undefined) window.clearTimeout(saveTimer);
  const save = () => {
    saveTimer = undefined;
    const currentComicId = loadedComicId;
    const currentServerChapterId = loadedChapterId;
    const page = savedPage + 1;
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
}

const setCurrentPage = (index: number): void => {
  if (index < 0 || index >= pages.value.length || index === currentPage.value)
    return;
  currentPage.value = index;
  savedPage = index;
  saveProgress();
}

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
}

const handleImageLoad = (index: number): void => {
  loadedPages.value[index] = true;
  handleScroll();
}

const chapterOffset = (offset: number): void => {
  const chapters = comic.value?.chapters ?? [];
  const index = chapters.findIndex((chapter) => chapter.id === chapterId.value);
  const next = chapters[index + offset];
  if (next) {
    savedPage = currentPage.value;
    saveProgress(true);
    void router.replace({
      name: "comic-reader",
      params: { comicId: comicId.value, chapterId: next.id },
    });
  }
}

/** 阅读页只持有分卷 ID；返回时复用来源页，避免将其误作系列标题。 */
const returnToDetail = (): void => {
  if (window.history.state?.back) router.back();
  else void router.push({ name: "comic" });
}

const changePage = (offset: number): void => {
  const next = currentPage.value + offset;
  if (next < 0 || next >= pages.value.length) {
    chapterOffset(offset);
    return;
  }
  currentPage.value = next;
  savedPage = next;
  saveProgress();
  void loadPage(next);
}

const pageOffsetForSide = (side: "left" | "right"): number => {
  const leftOffset = settings.pageTurnDirection === "left-next" ? 1 : -1;
  return side === "left" ? leftOffset : -leftOffset;
}

const handleReaderClick = (event: MouseEvent): void => {
  if ((event.target as HTMLElement).closest("button, a")) return;
  if (settings.mode === "paged") {
    const start = window.innerWidth / 3;
    const end = start * 2;
    if (event.clientX < start) {
      changePage(pageOffsetForSide("left"));
      return;
    }
    if (event.clientX > end) {
      changePage(pageOffsetForSide("right"));
      return;
    }
  }
  settingsVisible.value = true;
}

const saveCurrentProgress = (): void => {
  savedPage = currentPage.value;
  saveProgress(true);
}

const saveBeforeInterruption = (): void => {
  if (document.visibilityState === "hidden") saveCurrentProgress();
}

onMounted(() => {
  document.addEventListener("visibilitychange", saveBeforeInterruption);
  window.addEventListener("pagehide", saveCurrentProgress);
  window.addEventListener("scroll", handleScroll, { passive: true });
  void load();
});

onBeforeUnmount(() => {
  loadRequest += 1;
  loadObserver?.disconnect();
  savedPage = currentPage.value;
  saveProgress(true);
  document.removeEventListener("visibilitychange", saveBeforeInterruption);
  window.removeEventListener("pagehide", saveCurrentProgress);
  window.removeEventListener("scroll", handleScroll);
  if (scrollFrame !== undefined) cancelAnimationFrame(scrollFrame);
});

watch([comicId, chapterId], () => void load());

watch(
  () => settings.mode,
  (mode) => {
    currentPage.value = Math.max(0, savedPage);
    loadObserver?.disconnect();
    progressTrackingReady = false;
    if (mode === "scroll")
      void activateScrollReader(currentPage.value);
    else void loadPage(currentPage.value);
  },
);
</script>
<template>
  <ReaderToolbar
    :title="comic?.title || '漫画阅读'"
    :settings-disabled="loading"
    @back="returnToDetail"
    @settings="settingsVisible = true"
  />
  <article
    class="book-reader comic-reader"
    :class="[`book-reader--${settings.mode}`]"
    @click="handleReaderClick"
  >
    <LoadingOverlay
      v-if="loading"
      visible
      label="正在加载漫画章节"
    />
    <ErrorState
      v-if="error"
      title="漫画页面加载失败"
      :message="error"
      :loading="loading"
      @retry="load"
    />
    <ReaderBoundarySwitch
      v-else
      :has-previous="hasPreviousChapter"
      :has-next="hasNextChapter"
      :disabled="loading"
      @previous="chapterOffset(-1)"
      @next="chapterOffset(1)"
    >
      <section ref="pageList" class="comic-reader-pages">
        <p v-if="!loading && pages.length === 0" class="comic-reader-pending">
          正在获取章节页码…
        </p>
        <template v-if="settings.mode === 'scroll'">
          <div
            v-for="(page, index) in pages"
            :key="index"
            class="comic-reader-page"
            :class="{ 'comic-reader-page--loaded': loadedPages[index] }"
            :data-page-index="index"
          >
            <img
              v-if="page"
              class="comic-reader-page-image"
              :src="page"
              :alt="`第 ${index + 1} 页`"
              @load="handleImageLoad(index)"
            />
            <span v-else class="comic-reader-page-placeholder">
              加载第 {{ index + 1 }} 页…
            </span>
          </div>
        </template>
        <template v-else-if="pages.length">
          <div
            class="comic-reader-page"
            :class="{ 'comic-reader-page--loaded': loadedPages[currentPage] }"
            :data-page-index="currentPage"
          >
            <img
              v-if="pages[currentPage]"
              class="comic-reader-page-image"
              :src="pages[currentPage]"
              :alt="`第 ${currentPage + 1} 页`"
              @load="handleImageLoad(currentPage)"
            />
            <span v-else class="comic-reader-page-placeholder">
              加载第 {{ currentPage + 1 }} 页…
            </span>
          </div>
        </template>
      </section>
    </ReaderBoundarySwitch>
    <nav v-if="settings.mode === 'paged' && pages.length" class="page-controls" aria-label="漫画翻页">
      <var-button text round :elevation="false" aria-label="上一页" :disabled="loading || (currentPage === 0 && !hasPreviousChapter)" @click="changePage(-1)"><var-icon name="chevron-left" /></var-button>
      <span class="page-status" role="status">{{ currentPage + 1 }} / {{ pages.length }}</span>
      <var-button text round :elevation="false" aria-label="下一页" :disabled="loading || (currentPage === pages.length - 1 && !hasNextChapter)" @click="changePage(1)"><var-icon name="chevron-right" /></var-button>
    </nav>
    <ReaderSettingsDrawer
      v-model="settingsVisible"
      kind="comic"
      :title="comic?.title || '漫画阅读'"
      :previous-disabled="loading || !hasPreviousChapter"
      :next-disabled="loading || !hasNextChapter"
      @previous="chapterOffset(-1)"
      @next="chapterOffset(1)"
    />
  </article>
</template>
