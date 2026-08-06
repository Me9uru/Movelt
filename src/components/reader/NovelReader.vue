<script setup lang="ts">
import {
  ArrowRight,
  Reading,
  RefreshLeft,
} from "@element-plus/icons-vue";
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { ReaderDocument } from "../../domain/reader";
import type { ReadingProgress } from "../../domain/library";
import { useReaderSettings } from "../../composables/useReaderSettings";

const props = defineProps<{
  document: ReaderDocument;
  loading?: boolean;
  initialProgress?: ReadingProgress | null;
  hasNextChapter?: boolean;
}>();

const emit = defineEmits<{
  next: [];
  progress: [location: number];
}>();

const { settings, style, reset } = useReaderSettings();
const readerRoot = ref<HTMLElement | null>(null);
const pageViewport = ref<HTMLElement | null>(null);
const currentPage = ref(0);
const pageCount = ref(1);
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
  currentPage.value = Math.min(Math.max(page, 0), pageCount.value - 1);
  if (pageCount.value > 1) {
    pageLocation = currentPage.value / (pageCount.value - 1);
  }
  viewport.scrollTo({ left: currentPage.value * pageStep(), behavior: "smooth" });
  emit("progress", pageCount.value <= 1 ? 1 : pageLocation);
}

function scrollMetrics(): { start: number; distance: number } | null {
  const reader = readerRoot.value;
  if (!reader) return null;
  const start = reader.getBoundingClientRect().top + window.scrollY;
  return { start, distance: Math.max(1, reader.scrollHeight - window.innerHeight) };
}

function recordScrollProgress() {
  const metrics = scrollMetrics();
  if (!metrics) return;
  pageLocation = clampLocation((window.scrollY - metrics.start) / metrics.distance);
  emit("progress", pageLocation);
}

function handleScroll() {
  if (settings.mode !== "scroll") return;
  if (scrollTimer !== null) window.clearTimeout(scrollTimer);
  scrollTimer = window.setTimeout(() => {
    scrollTimer = null;
    recordScrollProgress();
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
    goToPage(currentPage.value - 1);
  } else if (event.clientX > middleEnd && settings.mode === "paged") {
    goToPage(currentPage.value + 1);
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
  hasRestoredPage = false;
  pageLocation = clampLocation(props.initialProgress?.location ?? 0);
  updatePagination(true);
});

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
  if (pageViewport.value) resizeObserver.observe(pageViewport.value);
  window.addEventListener("keydown", handleKeydown);
  window.addEventListener("scroll", handleScroll, { passive: true });
  updatePagination(true);
  restoreScrollProgress();
});

onBeforeUnmount(() => {
  cancelPaginationUpdate();
  spreadQuery?.removeEventListener("change", updateSpread);
  resizeObserver?.disconnect();
  window.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("scroll", handleScroll);
  if (scrollTimer !== null) window.clearTimeout(scrollTimer);
  if (settings.mode === "scroll") recordScrollProgress();
});
</script>

<template>
  <article
    ref="readerRoot"
    class="reader"
    :class="[`reader--${settings.theme}`, `reader--${settings.mode}`]"
    :style="style"
    @click="handleReaderClick"
  >
    <div v-if="settings.mode === 'scroll'" class="reader-body">
      <header class="reader-heading">
        <h1>{{ document.title }}</h1>
        <div class="reader-rule"><span></span><el-icon><Reading /></el-icon><span></span></div>
      </header>

      <div class="reader-content">
        <template v-for="(block, index) in document.blocks" :key="index">
          <p v-if="block.type === 'paragraph'">{{ block.text }}</p>
          <el-image v-else class="chapter-image" :src="block.url" :alt="block.alt || '小说插图'" fit="contain" />
        </template>
      </div>

      <el-divider>本章结束</el-divider>
      <el-button
        class="reader-next"
        type="primary"
        :icon="ArrowRight"
        :disabled="!hasNextChapter || loading"
        :title="hasNextChapter ? undefined : '已是最后一节'"
        round
        @click="$emit('next')"
      >
        下一节
      </el-button>
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
          <div class="reader-rule"><span></span><el-icon><Reading /></el-icon><span></span></div>
        </header>
        <template v-for="(block, index) in document.blocks" :key="index">
          <p v-if="block.type === 'paragraph'">{{ block.text }}</p>
          <img v-else class="chapter-image" :src="block.url" :alt="block.alt || '小说插图'" @load="updatePagination()" />
        </template>
        <p class="chapter-end">— 本章结束 —</p>
      </div>

      <nav class="page-controls" aria-label="分页状态与章节导航">
        <span class="page-status">{{ isSpread ? "双页" : "单页" }} · {{ pageLabel }}</span>
        <el-button
          v-if="currentPage >= pageCount - 1"
          class="paged-next"
          type="primary"
          :disabled="!hasNextChapter || loading"
          :title="hasNextChapter ? undefined : '已是最后一节'"
          round
          @click="$emit('next')"
        >
          下一节
        </el-button>
      </nav>
    </div>

    <el-drawer
      v-model="settingsVisible"
      class="reader-settings-drawer"
      direction="btt"
      size="min(680px, 78dvh)"
      :with-header="false"
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
  </article>
</template>
