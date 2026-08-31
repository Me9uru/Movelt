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
import ReaderSettingsDrawer from "../../components/reader/ReaderSettingsDrawer.vue";
import ReaderBoundarySwitch from "../../components/reader/ReaderBoundarySwitch.vue";
import LoadingOverlay from "../../components/common/LoadingOverlay.vue";
import {
  createReaderProgressSaver,
  readReaderProgress,
} from "../../utils/readerProgress";

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
const requestedBatches = new Set<number>();
let loadObserver: IntersectionObserver | undefined;
let progressObserver: IntersectionObserver | undefined;
let savedPage = -1;
let saveTimer: number | undefined;
// Vue Router 会在组件卸载前更新 route；保存进度必须使用本次已加载的
// 官方数字分卷 ID，而不能在卸载钩子中重新读取路由参数。
let loadedComicId = "";
let loadedChapterId = "";
let loadedRouteChapterId = "";
const persistProgress = createReaderProgressSaver("comic", (value) =>
  showError(value, "保存阅读进度失败"),
);

const loadPage = async (index: number): Promise<void> => {
  if (pages.value[index]) return;
  const batchStart = Math.floor(index / 12) * 12;
  if (requestedBatches.has(batchStart)) return;
  requestedBatches.add(batchStart);
  try {
    const batch = await getComicChapterPages(
      comicId.value,
      chapterId.value,
      index,
    );
    batch.pageUrls.forEach((url, offset) => {
      if (batch.startIndex + offset < pages.value.length)
        pages.value[batch.startIndex + offset] = url;
    });
  } catch (value) {
    error.value = getErrorMessage(value, "无法加载漫画页面");
    showError(value, "无法加载漫画页面");
  } finally {
    requestedBatches.delete(batchStart);
  }
}

const load = async (): Promise<void> => {
  // Keep the checkpoint for this load even if its cloud save completes while
  // the concurrently fetched detail still contains the previous position.
  const checkpoint = readReaderProgress("comic", comicId.value);
  loading.value = true;
  error.value = "";
  requestedBatches.clear();
  if (saveTimer !== undefined) window.clearTimeout(saveTimer);
  saveTimer = undefined;
  savedPage = -1;
  currentPage.value = 0;
  const pagesRequest = getComicChapterPages(
    comicId.value,
    chapterId.value,
    0,
  );
  const detailRequest = getComicBook(comicId.value);
  try {
    const [list, detail] = await Promise.all([pagesRequest, detailRequest]);
    comic.value = detail;
    loadedComicId = detail.id;
    loadedChapterId = list.chapterId;
    loadedRouteChapterId = chapterId.value;
    pages.value = Array.from(
      { length: list.pageCount },
      (_, index) => list.pageUrls[index] || "",
    );
    const position = list.readPosition ?? detail.readPosition;
    const page =
      checkpoint?.routeChapterId === chapterId.value &&
      checkpoint.serverChapterId === list.chapterId
        ? Number.parseInt(checkpoint.position, 10)
        : position?.chapterId === chapterId.value
          ? Number.parseInt(position.position, 10)
          : 1;
    savedPage = Math.min(
      Math.max(Number.isFinite(page) ? page - 1 : 0, 0),
      Math.max(0, pages.value.length - 1),
    );
    currentPage.value = savedPage;
    await nextTick();
    await loadPage(currentPage.value);
    await nextTick();
    observePages();
    if (settings.mode === "scroll")
      pageList.value
        ?.querySelector<HTMLElement>(`[data-page-index="${currentPage.value}"]`)
        ?.scrollIntoView({ block: "start", behavior: "auto" });
  } catch (value) {
    error.value = getErrorMessage(value, "无法加载漫画页面");
    showError(value, "无法加载漫画页面");
  } finally {
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
    const currentRouteChapterId = loadedRouteChapterId;
    const currentServerChapterId = loadedChapterId;
    const page = savedPage + 1;
    persistProgress(
      {
        itemId: currentComicId,
        routeChapterId: currentRouteChapterId,
        serverChapterId: currentServerChapterId,
        position: String(page),
      },
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
  progressObserver?.disconnect();
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
  progressObserver = new IntersectionObserver(
    (entries) => {
      const visible = entries
        .filter((entry) => entry.isIntersecting)
        .map((entry) => Number((entry.target as HTMLElement).dataset.pageIndex))
        .filter(Number.isInteger);
      if (visible.length) setCurrentPage(Math.min(...visible));
    },
    { rootMargin: "0px 0px -50%", threshold: 0.01 },
  );
  pageList.value
    ?.querySelectorAll<HTMLElement>(".comic-reader-page")
    .forEach((page) => {
      loadObserver?.observe(page);
      progressObserver?.observe(page);
    });
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
  void load();
});

onBeforeUnmount(() => {
  loadObserver?.disconnect();
  progressObserver?.disconnect();
  savedPage = currentPage.value;
  saveProgress(true);
  document.removeEventListener("visibilitychange", saveBeforeInterruption);
  window.removeEventListener("pagehide", saveCurrentProgress);
});

watch(chapterId, () => void load());

watch(
  () => settings.mode,
  () => {
    currentPage.value = Math.max(0, savedPage);
    void nextTick(() => {
      observePages();
      if (settings.mode === "paged") void loadPage(currentPage.value);
    });
  },
);
</script>
<template>
  <article
    class="book-reader comic-reader"
    :class="[`book-reader--${settings.mode}`]"
    @click="handleReaderClick"
  >
    <LoadingOverlay v-if="loading" inline visible label="正在加载漫画章节" />
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
        <template v-for="(page, index) in pages" :key="index">
          <img
            v-if="page"
            class="comic-reader-page"
            :data-page-index="index"
            :src="page"
            :alt="`第 ${index + 1} 页`"
          />
          <div
            v-else
            class="comic-reader-page comic-reader-page-placeholder"
            :data-page-index="index"
          >
            加载第 {{ index + 1 }} 页…
          </div>
        </template>
      </template>
      <template v-else-if="pages.length">
        <img
          v-if="pages[currentPage]"
          class="comic-reader-page"
          :data-page-index="currentPage"
          :src="pages[currentPage]"
          :alt="`第 ${currentPage + 1} 页`"
        />
        <div
          v-else
          class="comic-reader-page comic-reader-page-placeholder"
          :data-page-index="currentPage"
        >
          加载第 {{ currentPage + 1 }} 页…
        </div>
      </template>
      </section>
    </ReaderBoundarySwitch>
    <ReaderSettingsDrawer
      v-model="settingsVisible"
      kind="comic"
      :title="comic?.title || '漫画阅读'"
      :title-click="returnToDetail"
      :previous-disabled="
        !hasPreviousChapter
      "
      :next-disabled="
        !hasNextChapter
      "
      @previous="chapterOffset(-1)"
      @next="chapterOffset(1)"
    />
  </article>
</template>
