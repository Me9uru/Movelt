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
import type { NovelChapterContent as NovelChapterContentDto } from "../../domain/novel";
import { useReaderSettings } from "../../composables/reader/useReaderSettings";
import ReaderToolbar from "../../components/reader/ReaderToolbar.vue";
import ReaderSettingsDrawer from "../../components/reader/ReaderSettingsDrawer.vue";
import ReaderBoundarySwitch from "../../components/reader/ReaderBoundarySwitch.vue";
import NovelChapterContent from "../../components/reader/NovelChapterContent.vue";
import LoadingOverlay from "../../components/common/LoadingOverlay.vue";
import ErrorState from "../../components/common/ErrorState.vue";
import { getReaderDocument, getReaderOverview, lightNovelSourceId, saveReadPosition } from "../../services/novel";
import { getErrorMessage, showError } from "../../utils/error";
import { createReaderProgressSaver } from "../../utils/readerProgress";

const route = useRoute();
const router = useRouter();
const readerDocument = ref<NovelChapterContentDto | null>(null);
const resumePosition = ref<{ chapterId: string; position: string } | null>(null);
const chapterIds = ref<string[]>([]);
const loading = ref(true);
const error = ref("");
const chapterEntry = ref<"default" | "next" | "previous">("default");
const bookId = computed(() => typeof route.params.bookId === "string" ? route.params.bookId : "");
const chapterId = computed(() => typeof route.params.chapterId === "string" ? route.params.chapterId : "");
const chapterIndex = computed(() => chapterIds.value.indexOf(chapterId.value));
const hasPreviousChapter = computed(() => chapterIndex.value > 0);
const hasNextChapter = computed(() => chapterIndex.value >= 0 && chapterIndex.value < chapterIds.value.length - 1);

const { settings, style } = useReaderSettings("novel");
const readerRoot = ref<HTMLElement | null>(null);
const pageViewport = ref<HTMLElement | null>(null);
const readerContent = ref<HTMLElement | null>(null);
const currentPage = ref(0);
const pageCount = ref(1);
const previewVisible = ref(false);
const isSpread = ref(false);
const settingsVisible = ref(false);
let resizeObserver: ResizeObserver | undefined;
let spreadQuery: MediaQueryList | undefined;
let pointerStartX: number | null = null;
let suppressReaderClickUntil = 0;
let scrollTimer: number | null = null;
let paginationFrame: number | null = null;
let paginationRequest = 0;
let paginationResetPending = false;
let pageLocation = 0;
let hasRestoredScroll = false;
let hasRestoredPage = false;
let hasRestoredServerPosition = false;
let nextChapterRequested = false;
let previousChapterRequested = false;
const saveProgress = createReaderProgressSaver("novel", (value) =>
  showError(value, "保存阅读进度失败"),
);

const load = async (): Promise<void> => {
  if (!bookId.value || !chapterId.value) return;
  loading.value = true;
  error.value = "";
  readerDocument.value = null;
  try {
    const overview = await getReaderOverview(lightNovelSourceId, bookId.value);
    chapterIds.value = overview.chapters.map((chapter) => chapter.id);
    resumePosition.value = overview.readPosition;
    const document = await getReaderDocument(
      lightNovelSourceId,
      bookId.value,
      chapterId.value,
      settings.convert,
    );
    readerDocument.value = document;
  } catch (value) {
    error.value = getErrorMessage(value, "无法加载章节");
    showError(value, "无法加载章节");
  } finally {
    loading.value = false;
  }
}

const recordProgress = (): void => {
  if (!readerDocument.value || !bookId.value) return;
  const xpath = visibleXPath();
  resumePosition.value = { chapterId: chapterId.value, position: xpath };
  const currentBookId = bookId.value;
  const currentServerChapterId = readerDocument.value.serverChapterId;
  saveProgress(
    currentBookId,
    () => saveReadPosition(currentBookId, currentServerChapterId, xpath),
  );
}

const changeChapter = (offset: number): void => {
  const nextId = chapterIds.value[chapterIndex.value + offset];
  if (!nextId) return;
  recordProgress();
  chapterEntry.value = offset > 0 ? "next" : "previous";
  void router.replace({
    name: "novel-reader",
    params: { bookId: bookId.value, chapterId: nextId },
    query: route.query,
  });
}

const saveBeforeInterruption = (): void => {
  if (document.visibilityState === "hidden") recordProgress();
}

const pageLabel = computed(
  () => `${currentPage.value + 1} / ${pageCount.value}`,
);

const updateSpread = () => {
  isSpread.value = Boolean(spreadQuery?.matches);
}

const pageStep = (): number => {
  const viewport = pageViewport.value;
  if (!viewport) return 0;
  const viewportStyle = getComputedStyle(viewport);
  const gap = Number.parseFloat(viewportStyle.columnGap) || 0;
  const padding =
    (Number.parseFloat(viewportStyle.paddingLeft) || 0) +
    (Number.parseFloat(viewportStyle.paddingRight) || 0);
  return Math.max(1, viewport.clientWidth - padding + gap);
}

const clampLocation = (location: number): number => {
  return Math.min(1, Math.max(0, location));
}

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
  if (resetPage && !hasRestoredPage) {
    pageLocation =
      chapterEntry.value !== "default"
        ? 0
        : pageLocation;
  }
  currentPage.value = Math.round(
    pageLocation * Math.max(0, pageCount.value - 1),
  );
  hasRestoredPage = true;
  viewport.scrollTo({ left: currentPage.value * step, behavior: "auto" });
  restoreServerPosition();
}

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
}

const setReaderContent = (element: HTMLElement): void => {
  readerContent.value = element;
}

const cancelPaginationUpdate = () => {
  paginationRequest++;
  paginationResetPending = false;
  if (paginationFrame !== null) {
    window.cancelAnimationFrame(paginationFrame);
    paginationFrame = null;
  }
}

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
    pageLocation = currentPage.value / (pageCount.value - 1);
  }
  viewport.scrollTo({
    left: currentPage.value * pageStep(),
    behavior: "smooth",
  });
  recordProgress();
}

const pageOffsetForSide = (side: "left" | "right"): number => {
  const leftOffset = settings.pageTurnDirection === "left-next" ? 1 : -1;
  return side === "left" ? leftOffset : -leftOffset;
}

const scrollMetrics = (): { start: number; distance: number } | null => {
  const reader = readerRoot.value;
  if (!reader) return null;
  const start = reader.getBoundingClientRect().top + window.scrollY;
  return {
    start,
    distance: Math.max(1, reader.scrollHeight - window.innerHeight),
  };
}

const recordScrollProgress = (): number | null => {
  const metrics = scrollMetrics();
  if (!metrics) return null;
  pageLocation = clampLocation(
    (window.scrollY - metrics.start) / metrics.distance,
  );
  recordProgress();
  return pageLocation;
}

const visibleXPath = (): string => {
  const root = readerContent.value;
  if (!root) return "//*";
  const nodes = [
    ...root.querySelectorAll<HTMLElement>("p, img, li, h1, h2, h3, blockquote"),
  ];
  const viewport = pageViewport.value;
  const target =
    settings.mode === "paged" && viewport
      ? (() => {
          const bounds = viewport.getBoundingClientRect();
          // CSS 多栏分页下，每段都处于相同的纵向坐标；要按横向视口选择当前页的首个元素。
          return nodes.find((node) => {
            const rect = node.getBoundingClientRect();
            return rect.right > bounds.left && rect.left < bounds.right;
          });
        })()
      : nodes.find((node) => node.getBoundingClientRect().bottom >= 0);
  if (!target) return "//*";
  const path: string[] = [];
  let current: Element | null = target;
  while (current && current !== root) {
    const siblings = [...(current.parentElement?.children ?? [])].filter(
      (item) => item.tagName === current!.tagName,
    );
    path.unshift(
      `${current.tagName.toLowerCase()}[${siblings.indexOf(current) + 1}]`,
    );
    current = current.parentElement;
  }
  return `//*${path.length ? `/${path.join("/")}` : ""}`;
}

const restoreServerPosition = () => {
  if (hasRestoredServerPosition || chapterEntry.value !== "default") return;
  const position =
    resumePosition.value?.chapterId === readerDocument.value?.chapterId
      ? resumePosition.value
      : readerDocument.value?.readPosition?.chapterId ===
          readerDocument.value?.serverChapterId
        ? readerDocument.value?.readPosition ?? null
        : null;
  const root = readerContent.value;
  if (!position || !root || !position.position) return;
  try {
    const target = document.evaluate(
      position.position,
      root,
      null,
      XPathResult.FIRST_ORDERED_NODE_TYPE,
      null,
    ).singleNodeValue;
    if (!(target instanceof HTMLElement)) return;
    target.scrollIntoView({ block: "start", behavior: "auto" });
    if (settings.mode === "paged" && pageViewport.value) {
      currentPage.value = Math.round(
        pageViewport.value.scrollLeft / pageStep(),
      );
      pageLocation =
        pageCount.value > 1 ? currentPage.value / (pageCount.value - 1) : 0;
    }
    hasRestoredServerPosition = true;
  } catch {
    // Old or malformed server positions must not block chapter rendering.
  }
}

const requestNextChapter = () => {
  if (nextChapterRequested || loading.value || !hasNextChapter.value) return;
  nextChapterRequested = true;
  changeChapter(1);
}

const requestPreviousChapter = () => {
  if (previousChapterRequested || loading.value || !hasPreviousChapter.value)
    return;
  previousChapterRequested = true;
  changeChapter(-1);
}

const handleScroll = () => {
  if (settings.mode !== "scroll") return;
  if (scrollTimer !== null) window.clearTimeout(scrollTimer);
  scrollTimer = window.setTimeout(() => {
    scrollTimer = null;
    if (recordScrollProgress() === null) return;
  }, 120);
}

const restoreScrollProgress = () => {
  if (settings.mode !== "scroll" || hasRestoredScroll) return;
  void nextTick(() => {
    const metrics = scrollMetrics();
    if (!metrics) return;
    hasRestoredScroll = true;
    window.scrollTo({
      top: metrics.start + pageLocation * metrics.distance,
      behavior: "auto",
    });
  });
}

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
    goToPage(currentPage.value + pageOffsetForSide("left"));
  } else if (event.key === "ArrowRight" || event.key === " ") {
    event.preventDefault();
    goToPage(currentPage.value + (event.key === " " ? 1 : pageOffsetForSide("right")));
  }
}

const handlePointerDown = (event: PointerEvent) => {
  if (previewVisible.value) return;
  pointerStartX = event.clientX;
}

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
}

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
    goToPage(currentPage.value + pageOffsetForSide("left"));
  } else if (event.clientX > middleEnd && settings.mode === "paged") {
    goToPage(currentPage.value + pageOffsetForSide("right"));
  } else if (event.clientX >= middleStart && event.clientX <= middleEnd) {
    settingsVisible.value = true;
  }
}

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

watch(
  () => readerDocument.value,
  () => {
    nextChapterRequested = false;
    previousChapterRequested = false;
    hasRestoredPage = false;
    hasRestoredScroll = false;
    hasRestoredServerPosition = chapterEntry.value !== "default";
    pageLocation =
      chapterEntry.value !== "default"
        ? 0
        : 0;
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
      hasRestoredPage = false;
    }
  },
);

onMounted(() => {
  spreadQuery = window.matchMedia("(min-width: 960px)");
  updateSpread();
  spreadQuery.addEventListener("change", updateSpread);
  resizeObserver = new ResizeObserver(() => updatePagination());
  if (pageViewport.value) resizeObserver.observe(pageViewport.value);
  window.addEventListener("keydown", handleKeydown);
  window.addEventListener("scroll", handleScroll, { passive: true });
  document.addEventListener("visibilitychange", saveBeforeInterruption);
  window.addEventListener("pagehide", recordProgress);
  updatePagination(true);
  restoreScrollProgress();
  if (settings.mode === "scroll") void nextTick(restoreServerPosition);
  void load();
});

watch(chapterId, () => void load());
watch(() => settings.convert, () => {
  if (readerDocument.value) {
    recordProgress();
    void load();
  }
});

onBeforeUnmount(() => {
  // 路由返回会立即销毁阅读器；此时仍可从挂载的正文取到最后可见锚点。
  // 不能只依赖滚动事件或翻页操作，否则分页模式直接返回会漏掉最后进度。
  recordProgress();
  cancelPaginationUpdate();
  spreadQuery?.removeEventListener("change", updateSpread);
  resizeObserver?.disconnect();
  window.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("scroll", handleScroll);
  document.removeEventListener("visibilitychange", saveBeforeInterruption);
  window.removeEventListener("pagehide", recordProgress);
  if (scrollTimer !== null) window.clearTimeout(scrollTimer);
});
</script>

<template>
  <ReaderToolbar
    :title="readerDocument?.title || '小说阅读'"
    :settings-disabled="loading || !readerDocument"
    @back="router.push({ name: 'novel-detail', params: { bookId }, query: route.query })"
    @settings="settingsVisible = true"
  />
  <LoadingOverlay v-if="loading && !readerDocument" inline visible label="正在加载章节" />
  <ErrorState v-else-if="error" title="章节加载失败" :message="error" :loading="loading" @retry="load" />
  <article
    v-else-if="readerDocument"
    ref="readerRoot"
    class="book-reader"
    :class="[
      `book-reader--${settings.mode}`,
      { 'book-reader--chapter-font': Boolean(readerDocument.fontUrl) },
    ]"
    :style="style"
    @click="handleReaderClick"
  >
    <ReaderBoundarySwitch
      v-if="settings.mode === 'scroll'"
      :has-previous="hasPreviousChapter"
      :has-next="hasNextChapter"
      :disabled="loading"
      @previous="requestPreviousChapter"
      @next="requestNextChapter"
    >
      <div class="reader-body">
        <NovelChapterContent
          :document="readerDocument"
          heading-class="reader-heading"
          @ready="setReaderContent"
          @layout-change="updatePagination(true)"
          @preview-visible="previewVisible = $event"
        />

        <var-divider>本章结束</var-divider>
      </div>
    </ReaderBoundarySwitch>

    <div
      v-else
      class="paged-reader"
      :class="{ 'paged-reader--spread': isSpread }"
    >
      <div
        ref="pageViewport"
        class="page-viewport"
        tabindex="0"
        aria-label="分页阅读区域，可使用左右方向键翻页"
        @pointerdown="handlePointerDown"
        @pointerup="handlePointerUp"
        @pointercancel="pointerStartX = null"
      >
        <NovelChapterContent
          :document="readerDocument"
          heading-class="paged-heading"
          @ready="setReaderContent"
          @layout-change="updatePagination(true)"
          @preview-visible="previewVisible = $event"
        />
        <p class="chapter-end">— 本章结束 —</p>
      </div>

      <nav class="page-controls" aria-label="分页状态与章节导航">
        <var-button text round :elevation="false" aria-label="上一页" :disabled="loading || (currentPage === 0 && !hasPreviousChapter)" @click="goToPage(currentPage - 1)"><var-icon name="chevron-left" /></var-button>
        <span class="page-status" role="status">{{ isSpread ? "双页" : "单页" }} · {{ pageLabel }}</span>
        <var-button text round :elevation="false" aria-label="下一页" :disabled="loading || (currentPage >= pageCount - 1 && !hasNextChapter)" @click="goToPage(currentPage + 1)"><var-icon name="chevron-right" /></var-button>
      </nav>
    </div>

    <ReaderSettingsDrawer
      v-model="settingsVisible"
      kind="novel"
      :title="readerDocument.title"
      :previous-disabled="!hasPreviousChapter || loading"
      :next-disabled="!hasNextChapter || loading"
      @previous="changeChapter(-1)"
      @next="changeChapter(1)"
    />

  </article>
</template>
