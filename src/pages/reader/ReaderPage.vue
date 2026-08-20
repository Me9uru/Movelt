<script setup lang="ts">
import {
  ArrowLeft,
  ArrowRight,
  RefreshLeft,
} from "@element-plus/icons-vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { ReaderDocument } from "../../domain/reader";
import { useReaderSettings } from "../../composables/useReaderSettings";
import { clearWebviewCache } from "../../services/settings";

const props = defineProps<{
  document: ReaderDocument;
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

const { settings, style, reset } = useReaderSettings();
const readerRoot = ref<HTMLElement | null>(null);
const pageViewport = ref<HTMLElement | null>(null);
const readerContent = ref<HTMLElement | null>(null);
const currentPage = ref(0);
const pageCount = ref(1);
const previewImageUrl = ref<string | null>(null);
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
let nextChapterRequested = false;
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

const pageLabel = computed(() => `${currentPage.value + 1} / ${pageCount.value}`);

function updateSpread() {
  isSpread.value = Boolean(spreadQuery?.matches);
}

function pageStep(): number {
  const viewport = pageViewport.value;
  if (!viewport) return 0;
  const viewportStyle = getComputedStyle(viewport);
  const gap = Number.parseFloat(viewportStyle.columnGap) || 0;
  const padding = (Number.parseFloat(viewportStyle.paddingLeft) || 0)
    + (Number.parseFloat(viewportStyle.paddingRight) || 0);
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
  const scrollDistance = Math.max(0, viewport.scrollWidth - viewport.clientWidth);
  pageCount.value = Math.max(1, Math.floor(scrollDistance / step) + 1);
  if (resetPage && !hasRestoredPage) {
    pageLocation = clampLocation(props.initialProgress?.location ?? pageLocation);
  }
  currentPage.value = Math.round(pageLocation * Math.max(0, pageCount.value - 1));
  hasRestoredPage = true;
  viewport.scrollTo({ left: currentPage.value * step, behavior: "auto" });
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
  content.querySelectorAll("img").forEach((image) => contentResizeObserver?.observe(image));
}

function prepareFootnotes(content: HTMLElement): void {
  content.querySelectorAll<HTMLAnchorElement>("a.duokan-footnote").forEach((footnote) => {
    if (footnote.dataset.movelFootnoteReady) return;
    footnote.dataset.movelFootnoteReady = "true";

    const targetId = footnote.getAttribute("href")?.replace(/^#/, "");
    if (!targetId) return;
    const note = content.querySelector<HTMLElement>(`#${CSS.escape(targetId)}`);
    if (!note) return;

    note.hidden = true;
    footnote.removeAttribute("href");
    footnote.querySelectorAll<HTMLImageElement>("img.footnote").forEach((image) => {
      image.replaceWith(document.createTextNode("*"));
    });
    footnote.setAttribute("aria-label", "查看注释");
    footnote.title = note.textContent?.trim() || "查看注释";
    footnote.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      void ElMessageBox.alert(note.innerHTML, "注释", {
        dangerouslyUseHTMLString: true,
        confirmButtonText: "关闭",
      });
    });
  });
}

function handleChapterImageLoad(event: Event) {
  if (event.target instanceof HTMLImageElement) updatePagination();
}

function handleChapterImageClick(event: MouseEvent) {
  const image = event.target;
  if (!(image instanceof HTMLImageElement)) return;
  if (!image.closest(".illus, .illu, .duokan-image-single, .image-preview")) return;
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
    document.getElementById(href.slice(1))?.scrollIntoView({ block: "start", behavior: "smooth" });
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

async function clearBrowsingData(): Promise<void> {
  try {
    await clearWebviewCache();
    ElMessage.success("已清除 WebView 浏览数据");
  } catch {
    ElMessage.error("清除 WebView 数据失败");
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
  if (page < pageCount.value - 1) nextChapterRequested = false;
  currentPage.value = Math.min(Math.max(page, 0), pageCount.value - 1);
  if (pageCount.value > 1) {
    pageLocation = currentPage.value / (pageCount.value - 1);
  }
  viewport.scrollTo({ left: currentPage.value * pageStep(), behavior: "smooth" });
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
  return { start, distance: Math.max(1, reader.scrollHeight - window.innerHeight) };
}

function recordScrollProgress(): number | null {
  const metrics = scrollMetrics();
  if (!metrics) return null;
  pageLocation = clampLocation((window.scrollY - metrics.start) / metrics.distance);
  emit("progress", visibleXPath());
  return pageLocation;
}

function visibleXPath(): string {
  const root = readerContent.value;
  if (!root) return "//*";
  const target = [...root.querySelectorAll<HTMLElement>("p, img, li, h1, h2, h3, blockquote")]
    .find((node) => node.getBoundingClientRect().bottom >= 0);
  if (!target) return "//*";
  const path: string[] = [];
  let current: Element | null = target;
  while (current && current !== root) {
    const siblings = [...(current.parentElement?.children ?? [])].filter((item) => item.tagName === current!.tagName);
    path.unshift(`${current.tagName.toLowerCase()}[${siblings.indexOf(current) + 1}]`);
    current = current.parentElement;
  }
  return `//*${path.length ? `/${path.join("/")}` : ""}`;
}

function restoreServerPosition() {
  const position = props.document.readPosition;
  const root = readerContent.value;
  if (!position || !root || position.chapterId !== props.document.serverChapterId || !position.position) return;
  try {
    const target = document.evaluate(position.position, root, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
    if (target instanceof HTMLElement) target.scrollIntoView({ block: "start", behavior: "auto" });
  } catch {
    // Old or malformed server positions must not block chapter rendering.
  }
}

function requestNextChapter() {
  if (nextChapterRequested || props.loading || !props.hasNextChapter) return;
  nextChapterRequested = true;
  emit("next");
}

function handleScroll() {
  if (settings.mode !== "scroll") return;
  if (scrollTimer !== null) window.clearTimeout(scrollTimer);
  scrollTimer = window.setTimeout(() => {
    scrollTimer = null;
    const location = recordScrollProgress();
    if (location === null) return;
    if (location >= 0.995) requestNextChapter();
    else if (location < 0.98 && !props.loading) nextChapterRequested = false;
  }, 120);
}

function restoreScrollProgress() {
  if (settings.mode !== "scroll" || hasRestoredScroll) return;
  void nextTick(() => {
    const metrics = scrollMetrics();
    if (!metrics) return;
    hasRestoredScroll = true;
    window.scrollTo({
      top: metrics.start + (props.initialProgress?.location ?? 0) * metrics.distance,
      behavior: "auto",
    });
  });
}

function handleKeydown(event: KeyboardEvent) {
  if (settings.mode !== "paged") return;
  const target = event.target;
  if (target instanceof HTMLElement && target.closest("input, button, [contenteditable='true']")) return;
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    goToPage(currentPage.value - 1);
  } else if (event.key === "ArrowRight" || event.key === " ") {
    event.preventDefault();
    goToPage(currentPage.value + 1);
  }
}

function handlePointerDown(event: PointerEvent) {
  pointerStartX = event.clientX;
}

function handlePointerUp(event: PointerEvent) {
  if (pointerStartX === null) return;
  const distance = event.clientX - pointerStartX;
  pointerStartX = null;
  if (Math.abs(distance) < 45) return;
  suppressReaderClickUntil = performance.now() + 350;
  goToPage(currentPage.value + (distance < 0 ? 1 : -1));
}

function handleReaderClick(event: MouseEvent) {
  if (performance.now() < suppressReaderClickUntil) return;

  const target = event.target;
  if (target instanceof HTMLElement && target.closest(
    "button, a, input, label, [role='button'], [contenteditable='true'], .el-slider, .el-radio-group",
  )) return;
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
  () => [settings.mode, settings.fontSize, settings.lineHeight,
    settings.letterSpacing, settings.paragraphSpacing, settings.contentWidth,
    settings.font, isSpread.value],
  () => updatePagination(true),
  { deep: true },
);

watch(() => props.document, () => {
  nextChapterRequested = false;
  hasRestoredPage = false;
  pageLocation = clampLocation(props.initialProgress?.location ?? 0);
  void nextTick(() => {
    observeChapterContent();
    updatePagination(true);
    restoreServerPosition();
  });
}, { immediate: true });

watch(() => props.document.fontUrl, loadChapterFont, { immediate: true });

watch(() => settings.mode, (mode) => {
  if (mode === "scroll") {
    cancelPaginationUpdate();
    hasRestoredScroll = false;
    restoreScrollProgress();
  } else {
    hasRestoredPage = false;
  }
});

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
  void nextTick(restoreServerPosition);
});

onBeforeUnmount(() => {
  cancelPaginationUpdate();
  spreadQuery?.removeEventListener("change", updateSpread);
  resizeObserver?.disconnect();
  contentResizeObserver?.disconnect();
  window.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("scroll", handleScroll);
  if (scrollTimer !== null) window.clearTimeout(scrollTimer);
  chapterFontStyle.remove();
  if (settings.mode === "scroll") recordScrollProgress();
});
</script>

<template>
  <article
    ref="readerRoot"
    class="reader"
    :class="[
      `reader--${settings.theme}`,
      `reader--${settings.mode}`,
      { 'reader--chapter-font': Boolean(document.fontUrl) },
    ]"
    :style="style"
    @click="handleReaderClick"
  >
    <Teleport to="body">
      <Transition name="reader-chapter-nav">
        <nav v-if="settingsVisible" class="reader-chapter-nav" aria-label="章节导航">
          <el-button
            circle
            :icon="ArrowLeft"
            :disabled="!hasPreviousChapter || loading"
            :title="hasPreviousChapter ? '上一话' : '已是第一话'"
            aria-label="上一话"
            @click="emit('previous')"
          />
          <strong :title="document.title">{{ document.title }}</strong>
          <el-button
            circle
            :icon="ArrowRight"
            :disabled="!hasNextChapter || loading"
            :title="hasNextChapter ? '下一话' : '已是最后一话'"
            aria-label="下一话"
            @click="emit('next')"
          />
        </nav>
      </Transition>
    </Teleport>

    <div v-if="settings.mode === 'scroll'" class="reader-body">
      <header class="reader-heading">
        <h1>{{ document.title }}</h1>
      </header>

      <div ref="readerContent" class="reader-content" v-html="document.html" @click="handleChapterLinkClick" @click.capture="handleChapterImageClick" @load.capture="handleChapterImageLoad" />

      <el-divider>本章结束</el-divider>
    </div>

    <div v-else class="paged-reader" :class="{ 'paged-reader--spread': isSpread }">
      <div
        ref="pageViewport"
        class="page-viewport"
        tabindex="0"
        aria-label="分页阅读区域，可使用左右方向键翻页"
        @pointerdown="handlePointerDown"
        @pointerup="handlePointerUp"
        @pointercancel="pointerStartX = null"
      >
        <header class="paged-heading">
          <h1>{{ document.title }}</h1>
        </header>
        <div ref="readerContent" class="reader-content" v-html="document.html" @click="handleChapterLinkClick" @click.capture="handleChapterImageClick" @load.capture="handleChapterImageLoad" />
        <p class="chapter-end">— 本章结束 —</p>
      </div>

      <nav class="page-controls" aria-label="分页状态与章节导航">
        <span class="page-status">{{ isSpread ? "双页" : "单页" }} · {{ pageLabel }}</span>
      </nav>
    </div>

    <el-drawer
      v-model="settingsVisible"
      class="reader-settings-drawer"
      direction="btt"
      size="min(680px, calc(100dvh - env(safe-area-inset-top) - 138px))"
      :with-header="false"
      :z-index="2000"
      append-to-body
    >
      <div class="settings-panel">
        <div class="settings-handle" aria-hidden="true"></div>
        <div class="settings-title">
          <strong>阅读设置</strong>
          <el-button :icon="RefreshLeft" text size="small" @click="reset">恢复默认</el-button>
        </div>

        <label>阅读模式</label>
        <el-radio-group v-model="settings.mode" size="small">
          <el-radio-button value="scroll">滚动阅读</el-radio-button>
          <el-radio-button value="paged">分页阅读</el-radio-button>
        </el-radio-group>

        <template v-if="settings.mode === 'paged'">
          <label>翻页点击方向</label>
          <el-radio-group v-model="settings.pageTurnDirection" size="small">
            <el-radio-button value="left-previous">左边上一页 · 右边下一页</el-radio-button>
            <el-radio-button value="left-next">左边下一页 · 右边上一页</el-radio-button>
          </el-radio-group>
        </template>

        <label>背景主题</label>
        <el-radio-group v-model="settings.theme" size="small">
          <el-radio-button value="paper">纸张</el-radio-button>
          <el-radio-button value="light">明亮</el-radio-button>
          <el-radio-button value="night">夜间</el-radio-button>
        </el-radio-group>

        <label>正文字体</label>
        <el-radio-group v-model="settings.font" size="small">
          <el-radio-button value="serif">衬线</el-radio-button>
          <el-radio-button value="sans">无衬线</el-radio-button>
        </el-radio-group>

        <label>文字转换</label>
        <el-radio-group v-model="settings.convert" size="small">
          <el-radio-button value="original">原文</el-radio-button>
          <el-radio-button value="t2s">繁转简</el-radio-button>
          <el-radio-button value="s2t">简转繁</el-radio-button>
        </el-radio-group>

        <label>存储</label>
        <el-button size="small" @click="clearBrowsingData">清除 WebView 数据</el-button>

        <label><span>字体大小</span><b>{{ settings.fontSize }} px</b></label>
        <el-slider v-model="settings.fontSize" :min="14" :max="30" :step="1" />

        <label><span>行间距</span><b>{{ settings.lineHeight.toFixed(1) }} 倍</b></label>
        <el-slider v-model="settings.lineHeight" :min="1.4" :max="2.6" :step="0.1" />

        <label><span>字间距</span><b>{{ settings.letterSpacing.toFixed(1) }} px</b></label>
        <el-slider v-model="settings.letterSpacing" :min="0" :max="4" :step="0.2" />

        <label><span>段间距</span><b>{{ settings.paragraphSpacing.toFixed(1) }} 倍</b></label>
        <el-slider v-model="settings.paragraphSpacing" :min="0.6" :max="2.4" :step="0.1" />

        <label><span>阅读宽度</span><b>{{ settings.contentWidth }} px</b></label>
        <el-slider v-model="settings.contentWidth" :min="560" :max="1100" :step="20" />
      </div>
    </el-drawer>

    <el-image-viewer
      v-if="previewImageUrl"
      :url-list="[previewImageUrl]"
      @close="previewImageUrl = null"
    />
  </article>
</template>
