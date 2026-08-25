<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from "vue";
import type { ReaderDocument } from "../../domain/novel";
import { useReaderSettings } from "../../composables/useReaderSettings";
import ReaderSettingsDrawer from "../../components/reader/ReaderSettingsDrawer.vue";
import ReaderBoundarySwitch from "../../components/reader/ReaderBoundarySwitch.vue";

const props = defineProps<{
  document: ReaderDocument;
  resumePosition?: { chapterId: string; position: string } | null;
  chapterEntry?: "default" | "next" | "previous";
  loading?: boolean;
  initialProgress?: { location: number } | null;
  hasPreviousChapter?: boolean;
  hasNextChapter?: boolean;
}>();

const emit = defineEmits<{
  previous: [];
  next: [];
  progress: [xpath: string];
}>();

const { settings, style } = useReaderSettings("novel");
const readerRoot = ref<HTMLElement | null>(null);
const pageViewport = ref<HTMLElement | null>(null);
const readerContent = ref<HTMLElement | null>(null);
const currentPage = ref(0);
const pageCount = ref(1);
const previewImageUrl = ref<string | null>(null);
const footnoteHtml = ref("");
const footnoteVisible = ref(false);
const isSpread = ref(false);
const settingsVisible = ref(false);
let resizeObserver: ResizeObserver | undefined;
let contentResizeObserver: ResizeObserver | undefined;
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
const chapterFontStyle = document.createElement("style");
document.head.append(chapterFontStyle);

function loadChapterFont(fontUrl: string | null): void {
  chapterFontStyle.textContent = fontUrl
    ? `@font-face { font-family: "movel-chapter"; font-display: block; src: url(${JSON.stringify(fontUrl)}); }`
    : "";
  void document.fonts.ready.then(() => {
    observeChapterContent();
    updatePagination(true);
  });
}

const pageLabel = computed(
  () => `${currentPage.value + 1} / ${pageCount.value}`,
);
const hasLeadingDocumentHeading = computed(() => {
  const content = new DOMParser().parseFromString(
    props.document.html,
    "text/html",
  ).body;
  return /^H[1-6]$/.test(content.firstElementChild?.tagName ?? "");
});

function updateSpread() {
  isSpread.value = Boolean(spreadQuery?.matches);
}

function pageStep(): number {
  const viewport = pageViewport.value;
  if (!viewport) return 0;
  const viewportStyle = getComputedStyle(viewport);
  const gap = Number.parseFloat(viewportStyle.columnGap) || 0;
  const padding =
    (Number.parseFloat(viewportStyle.paddingLeft) || 0) +
    (Number.parseFloat(viewportStyle.paddingRight) || 0);
  return Math.max(1, viewport.clientWidth - padding + gap);
}

function clampLocation(location: number): number {
  return Math.min(1, Math.max(0, location));
}

function performPagination(resetPage: boolean) {
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
      props.chapterEntry !== "default"
        ? 0
        : clampLocation(props.initialProgress?.location ?? pageLocation);
  }
  currentPage.value = Math.round(
    pageLocation * Math.max(0, pageCount.value - 1),
  );
  hasRestoredPage = true;
  viewport.scrollTo({ left: currentPage.value * step, behavior: "auto" });
  restoreServerPosition();
}

function updatePagination(resetPage = false) {
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

function observeChapterContent() {
  const content = readerContent.value;
  if (!content) return;
  prepareFootnotes(content);
  contentResizeObserver?.disconnect();
  contentResizeObserver?.observe(content);
  content
    .querySelectorAll("img")
    .forEach((image) => contentResizeObserver?.observe(image));
}

function prepareFootnotes(content: HTMLElement): void {
  content
    .querySelectorAll<HTMLAnchorElement>("a.duokan-footnote")
    .forEach((footnote) => {
      if (footnote.dataset.movelFootnoteReady) return;
      footnote.dataset.movelFootnoteReady = "true";

      const targetId = footnote.getAttribute("href")?.replace(/^#/, "");
      if (!targetId) return;
      const note = content.querySelector<HTMLElement>(
        `#${CSS.escape(targetId)}`,
      );
      if (!note) return;

      note.hidden = true;
      footnote.removeAttribute("href");
      footnote
        .querySelectorAll<HTMLImageElement>("img.footnote")
        .forEach((image) => {
          image.replaceWith(document.createTextNode("*"));
        });
      footnote.setAttribute("aria-label", "查看注释");
      footnote.title = note.textContent?.trim() || "查看注释";
      footnote.addEventListener("click", (event) => {
        event.preventDefault();
        event.stopPropagation();
        // 章节 HTML 已在 Rust 边界清洗。保留注释片段的标记，避免实体和排版在弹窗中被当作纯文本显示。
        footnoteHtml.value = note.innerHTML || "暂无注释内容";
        footnoteVisible.value = true;
      });
    });
}

function handleChapterImageLoad(event: Event) {
  if (event.target instanceof HTMLImageElement) updatePagination();
}

function handleChapterImageClick(event: MouseEvent) {
  const image = event.target;
  if (!(image instanceof HTMLImageElement)) return;
  if (!image.closest(".illus, .illu, .duokan-image-single, .image-preview"))
    return;
  event.stopPropagation();
  previewImageUrl.value = image.currentSrc || image.src;
}

function handleChapterLinkClick(event: MouseEvent) {
  const target = event.target;
  if (!(target instanceof Element)) return;
  const link = target.closest<HTMLAnchorElement>("a[href]");
  if (!link) return;
  const href = link.getAttribute("href");
  if (!href) return;
  if (href.startsWith("#")) {
    event.preventDefault();
    document
      .getElementById(href.slice(1))
      ?.scrollIntoView({ block: "start", behavior: "smooth" });
    return;
  }
  try {
    const url = new URL(href, window.location.href);
    if (url.protocol !== "http:" && url.protocol !== "https:") return;
    event.preventDefault();
    window.open(url.href, "_blank", "noopener,noreferrer");
  } catch {
    event.preventDefault();
  }
}

function cancelPaginationUpdate() {
  paginationRequest++;
  paginationResetPending = false;
  if (paginationFrame !== null) {
    window.cancelAnimationFrame(paginationFrame);
    paginationFrame = null;
  }
}

function goToPage(page: number) {
  const viewport = pageViewport.value;
  if (!viewport) return;
  if (page >= pageCount.value) {
    emit("progress", visibleXPath());
    requestNextChapter();
    return;
  }
  if (page < 0) {
    emit("progress", visibleXPath());
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
  emit("progress", visibleXPath());
}

function pageOffsetForSide(side: "left" | "right"): number {
  const leftOffset = settings.pageTurnDirection === "left-next" ? 1 : -1;
  return side === "left" ? leftOffset : -leftOffset;
}

function scrollMetrics(): { start: number; distance: number } | null {
  const reader = readerRoot.value;
  if (!reader) return null;
  const start = reader.getBoundingClientRect().top + window.scrollY;
  return {
    start,
    distance: Math.max(1, reader.scrollHeight - window.innerHeight),
  };
}

function recordScrollProgress(): number | null {
  const metrics = scrollMetrics();
  if (!metrics) return null;
  pageLocation = clampLocation(
    (window.scrollY - metrics.start) / metrics.distance,
  );
  emit("progress", visibleXPath());
  return pageLocation;
}

function visibleXPath(): string {
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

function restoreServerPosition() {
  if (hasRestoredServerPosition || props.chapterEntry !== "default") return;
  const position =
    props.resumePosition?.chapterId === props.document.chapterId
      ? props.resumePosition
      : props.document.readPosition?.chapterId ===
          props.document.serverChapterId
        ? props.document.readPosition
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

function requestNextChapter() {
  if (nextChapterRequested || props.loading || !props.hasNextChapter) return;
  nextChapterRequested = true;
  emit("next");
}

function requestPreviousChapter() {
  if (previousChapterRequested || props.loading || !props.hasPreviousChapter)
    return;
  previousChapterRequested = true;
  emit("previous");
}

function handleScroll() {
  if (settings.mode !== "scroll") return;
  if (scrollTimer !== null) window.clearTimeout(scrollTimer);
  scrollTimer = window.setTimeout(() => {
    scrollTimer = null;
    if (recordScrollProgress() === null) return;
  }, 120);
}

function restoreScrollProgress() {
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

function handleKeydown(event: KeyboardEvent) {
  if (previewImageUrl.value) return;
  if (settings.mode !== "paged") return;
  const target = event.target;
  if (
    target instanceof HTMLElement &&
    target.closest("input, button, [contenteditable='true']")
  )
    return;
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    goToPage(currentPage.value - 1);
  } else if (event.key === "ArrowRight" || event.key === " ") {
    event.preventDefault();
    goToPage(currentPage.value + 1);
  }
}

function handlePointerDown(event: PointerEvent) {
  if (previewImageUrl.value) return;
  pointerStartX = event.clientX;
}

function handlePointerUp(event: PointerEvent) {
  if (previewImageUrl.value) {
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

function handleReaderClick(event: MouseEvent) {
  if (previewImageUrl.value) return;
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
  () => props.document,
  () => {
    nextChapterRequested = false;
    previousChapterRequested = false;
    hasRestoredPage = false;
    hasRestoredScroll = false;
    hasRestoredServerPosition = props.chapterEntry !== "default";
    pageLocation =
      props.chapterEntry !== "default"
        ? 0
        : clampLocation(props.initialProgress?.location ?? 0);
    void nextTick(() => {
      observeChapterContent();
      updatePagination(true);
      if (settings.mode === "scroll") restoreServerPosition();
    });
  },
  { immediate: true },
);

watch(() => props.document.fontUrl, loadChapterFont, { immediate: true });

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
  contentResizeObserver = new ResizeObserver(() => updatePagination());
  if (pageViewport.value) resizeObserver.observe(pageViewport.value);
  window.addEventListener("keydown", handleKeydown);
  window.addEventListener("scroll", handleScroll, { passive: true });
  updatePagination(true);
  void nextTick(observeChapterContent);
  restoreScrollProgress();
  if (settings.mode === "scroll") void nextTick(restoreServerPosition);
});

onBeforeUnmount(() => {
  // 路由返回会立即销毁阅读器；此时仍可从挂载的正文取到最后可见锚点。
  // 不能只依赖滚动事件或翻页操作，否则分页模式直接返回会漏掉最后进度。
  emit("progress", visibleXPath());
  cancelPaginationUpdate();
  spreadQuery?.removeEventListener("change", updateSpread);
  resizeObserver?.disconnect();
  contentResizeObserver?.disconnect();
  window.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("scroll", handleScroll);
  if (scrollTimer !== null) window.clearTimeout(scrollTimer);
  chapterFontStyle.remove();
});
</script>

<template>
  <article
    ref="readerRoot"
    class="book-reader"
    :class="[
      `book-reader--${settings.mode}`,
      { 'book-reader--chapter-font': Boolean(document.fontUrl) },
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
        <header v-if="!hasLeadingDocumentHeading" class="reader-heading">
          <h1>{{ document.title }}</h1>
        </header>

        <div
          ref="readerContent"
          class="reader-content"
          v-html="document.html"
          @click="handleChapterLinkClick"
          @click.capture="handleChapterImageClick"
          @load.capture="handleChapterImageLoad"
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
        <header v-if="!hasLeadingDocumentHeading" class="paged-heading">
          <h1>{{ document.title }}</h1>
        </header>
        <div
          ref="readerContent"
          class="reader-content"
          v-html="document.html"
          @click="handleChapterLinkClick"
          @click.capture="handleChapterImageClick"
          @load.capture="handleChapterImageLoad"
        />
        <p class="chapter-end">— 本章结束 —</p>
      </div>

      <nav class="page-controls" aria-label="分页状态与章节导航">
        <span class="page-status"
          >{{ isSpread ? "双页" : "单页" }} · {{ pageLabel }}</span
        >
      </nav>
    </div>

    <ReaderSettingsDrawer
      v-model="settingsVisible"
      kind="novel"
      :title="document.title"
      :previous-disabled="!hasPreviousChapter || loading"
      :next-disabled="!hasNextChapter || loading"
      @previous="emit('previous')"
      @next="emit('next')"
    />

    <var-image-preview
      :show="Boolean(previewImageUrl)"
      :images="previewImageUrl ? [previewImageUrl] : []"
      @update:show="previewImageUrl = null"
    />

    <var-dialog
      :show="footnoteVisible"
      title="注释"
      confirm-button-text="关闭"
      :dialog-class="
        document.fontUrl
          ? 'reader-footnote-dialog reader-footnote-dialog--chapter-font'
          : 'reader-footnote-dialog'
      "
      @update:show="footnoteVisible = $event"
      @confirm="footnoteVisible = false"
    >
      <div class="reader-footnote-content" v-html="footnoteHtml" />
    </var-dialog>
  </article>
</template>
