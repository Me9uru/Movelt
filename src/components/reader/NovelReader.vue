<script setup lang="ts">
import {
  ArrowLeft,
  ArrowRight,
  Reading,
  RefreshLeft,
  Setting,
} from "@element-plus/icons-vue";
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { ReaderDocument } from "../../domain/reader";
import type { ReadingProgress } from "../../domain/library";
import { useReaderSettings } from "../../composables/useReaderSettings";

const props = defineProps<{
  document: ReaderDocument;
  loading?: boolean;
  initialProgress?: ReadingProgress | null;
}>();

const emit = defineEmits<{
  back: [];
  progress: [location: number];
}>();

const { settings, style, reset } = useReaderSettings();
const readerRoot = ref<HTMLElement | null>(null);
const pageViewport = ref<HTMLElement | null>(null);
const currentPage = ref(0);
const pageCount = ref(1);
const isLandscape = ref(false);
let resizeObserver: ResizeObserver | undefined;
let orientationQuery: MediaQueryList | undefined;
let pointerStartX: number | null = null;
let scrollTimer: number | null = null;
let hasRestoredScroll = false;
let hasRestoredPage = false;

const pageLabel = computed(() => `${currentPage.value + 1} / ${pageCount.value}`);

function updateOrientation() {
  isLandscape.value = Boolean(orientationQuery?.matches);
}

function pageStep(): number {
  const viewport = pageViewport.value;
  if (!viewport) return 0;
  const gap = Number.parseFloat(getComputedStyle(viewport).columnGap) || 0;
  return viewport.clientWidth + gap;
}

function updatePagination(resetPage = false) {
  if (settings.mode !== "paged") return;
  const previousLocation = pageCount.value <= 1 ? 0 : currentPage.value / (pageCount.value - 1);
  void nextTick(() => {
    const viewport = pageViewport.value;
    if (!viewport) return;
    resizeObserver?.observe(viewport);

    const step = pageStep();
    pageCount.value = Math.max(1, Math.ceil((viewport.scrollWidth + 1) / Math.max(step, 1)));
    const location = resetPage && !hasRestoredPage
      ? props.initialProgress?.location ?? 0
      : previousLocation;
    currentPage.value = Math.round(location * Math.max(0, pageCount.value - 1));
    hasRestoredPage = true;
    viewport.scrollTo({ left: currentPage.value * step, behavior: "auto" });
  });
}

function goToPage(page: number) {
  const viewport = pageViewport.value;
  if (!viewport) return;
  currentPage.value = Math.min(Math.max(page, 0), pageCount.value - 1);
  viewport.scrollTo({ left: currentPage.value * pageStep(), behavior: "smooth" });
  emit("progress", pageCount.value <= 1 ? 1 : currentPage.value / (pageCount.value - 1));
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
  emit("progress", (window.scrollY - metrics.start) / metrics.distance);
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
  goToPage(currentPage.value + (distance < 0 ? 1 : -1));
}

watch(
  () => [props.document, settings.mode, settings.fontSize, settings.lineHeight,
    settings.letterSpacing, settings.paragraphSpacing, settings.contentWidth,
    settings.font, isLandscape.value],
  () => updatePagination(true),
  { deep: true },
);

watch(() => settings.mode, (mode) => {
  if (mode === "scroll") {
    hasRestoredScroll = false;
    restoreScrollProgress();
  } else {
    hasRestoredPage = false;
  }
});

onMounted(() => {
  orientationQuery = window.matchMedia("(orientation: landscape) and (min-width: 900px)");
  updateOrientation();
  orientationQuery.addEventListener("change", updateOrientation);
  resizeObserver = new ResizeObserver(() => updatePagination());
  if (pageViewport.value) resizeObserver.observe(pageViewport.value);
  window.addEventListener("keydown", handleKeydown);
  window.addEventListener("scroll", handleScroll, { passive: true });
  updatePagination(true);
  restoreScrollProgress();
});

onBeforeUnmount(() => {
  orientationQuery?.removeEventListener("change", updateOrientation);
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
    v-loading="loading"
    class="reader"
    :class="[`reader--${settings.theme}`, `reader--${settings.mode}`]"
    :style="style"
  >
    <div class="reader-toolbar" aria-label="阅读工具栏">
      <span v-if="settings.mode === 'paged'" class="page-status">
        {{ isLandscape ? "双页" : "单页" }} · {{ pageLabel }}
      </span>

      <el-popover placement="bottom-end" :width="340" trigger="click" popper-class="reader-settings-popover">
        <template #reference>
          <el-button :icon="Setting" round>阅读设置</el-button>
        </template>

        <div class="settings-panel">
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
      </el-popover>
    </div>

    <div v-if="settings.mode === 'scroll'" class="reader-body">
      <header class="reader-heading">
        <el-tag round effect="plain">正在阅读</el-tag>
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
      <el-button class="reader-back" :icon="ArrowLeft" round @click="$emit('back')">返回目录</el-button>
    </div>

    <div v-else class="paged-reader" :class="{ 'paged-reader--spread': isLandscape }">
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
          <span>正在阅读</span>
          <h1>{{ document.title }}</h1>
          <div class="reader-rule"><span></span><el-icon><Reading /></el-icon><span></span></div>
        </header>
        <template v-for="(block, index) in document.blocks" :key="index">
          <p v-if="block.type === 'paragraph'">{{ block.text }}</p>
          <img v-else class="chapter-image" :src="block.url" :alt="block.alt || '小说插图'" @load="updatePagination()" />
        </template>
        <p class="chapter-end">— 本章结束 —</p>
      </div>

      <nav class="page-controls" aria-label="分页导航">
        <el-button :icon="ArrowLeft" circle :disabled="currentPage === 0" aria-label="上一页" @click="goToPage(currentPage - 1)" />
        <span>{{ pageLabel }}</span>
        <el-button :icon="ArrowRight" circle :disabled="currentPage >= pageCount - 1" aria-label="下一页" @click="goToPage(currentPage + 1)" />
      </nav>
    </div>
  </article>
</template>

<style scoped>
.reader { min-height: calc(100vh - 76px); margin: 0 calc(50% - 50vw); padding: 18px max(24px, calc(50vw - 580px)) 80px; transition: background .25s, color .25s; }
.reader--paper { --reader-bg: #f4f0e7; --reader-surface: rgb(255 253 247 / 82%); --reader-text: #38352f; --reader-muted: #877f73; --reader-border: #ded6c8; background: var(--reader-bg); }
.reader--light { --reader-bg: #f7f7f7; --reader-surface: #fff; --reader-text: #292929; --reader-muted: #777; --reader-border: #e2e2e2; background: var(--reader-bg); }
.reader--night { --reader-bg: #191b1f; --reader-surface: #22252a; --reader-text: #c9c5bb; --reader-muted: #8f9196; --reader-border: #34373d; background: var(--reader-bg); color-scheme: dark; }
.reader-toolbar { position: sticky; top: 88px; z-index: 10; display: flex; align-items: center; justify-content: flex-end; gap: 10px; width: min(var(--reader-width), 100%); margin: 0 auto 34px; }
.reader-toolbar :deep(.el-button), .page-status { border: 1px solid var(--reader-border); color: var(--reader-text); background: color-mix(in srgb, var(--reader-surface) 92%, transparent); backdrop-filter: blur(12px); }
.page-status { padding: 8px 13px; border-radius: 999px; color: var(--reader-muted); font-size: 12px; }
.reader-body { width: min(var(--reader-width), 100%); margin: 0 auto; transition: width .2s; }
.reader-heading { margin: 0 auto 50px; text-align: center; }
.reader-heading h1, .paged-heading h1 { color: var(--reader-text); font-family: var(--reader-font-family); font-weight: 500; line-height: 1.4; }
.reader-heading h1 { margin: 22px 0 26px; font-size: clamp(30px, 5vw, 42px); }
.reader-rule { display: flex; align-items: center; justify-content: center; gap: 12px; color: #b39d81; }
.reader-rule span { width: 72px; height: 1px; background: var(--reader-border); }
.reader-content { padding: 48px 56px; border: 1px solid var(--reader-border); border-radius: 20px; background: var(--reader-surface); box-shadow: 0 22px 60px rgb(40 35 28 / 7%); }
.reader-content p, .page-viewport p { margin: 0 0 var(--reader-paragraph-spacing); color: var(--reader-text); font-family: var(--reader-font-family); font-size: var(--reader-font-size); line-height: var(--reader-line-height); letter-spacing: var(--reader-letter-spacing); text-align: justify; overflow-wrap: anywhere; }
.chapter-image { display: block; max-width: 100%; max-height: 80vh; margin: 36px auto; object-fit: contain; break-inside: avoid; }
.reader-body > :deep(.el-divider) { margin: 62px 0 34px; border-color: var(--reader-border); }
.reader-body > :deep(.el-divider .el-divider__text) { color: var(--reader-muted); background: var(--reader-bg); }
.reader-back { display: flex; margin: 0 auto; }

.reader--paged { height: calc(100dvh - 76px); min-height: 540px; padding-bottom: 20px; overflow: hidden; }
.reader--paged .reader-toolbar { position: relative; top: 0; margin-bottom: 16px; }
.paged-reader { width: min(var(--reader-width), 100%); margin: 0 auto; }
.page-viewport { height: calc(100dvh - 196px); min-height: 390px; padding: 42px 48px; overflow: hidden; outline: none; border: 1px solid var(--reader-border); border-radius: 20px; background: var(--reader-surface); box-shadow: 0 22px 60px rgb(40 35 28 / 7%); column-count: 1; column-gap: 64px; column-fill: auto; scroll-behavior: smooth; touch-action: pan-y; }
.paged-reader--spread { width: min(1100px, 100%); }
.paged-reader--spread .page-viewport { column-count: 2; column-gap: 72px; }
.paged-heading { margin-bottom: 42px; text-align: center; break-inside: avoid; }
.paged-heading > span { color: var(--reader-muted); font-size: 12px; letter-spacing: .12em; }
.paged-heading h1 { margin: 18px 0 22px; font-size: clamp(26px, 4vw, 38px); }
.page-viewport .chapter-image { max-height: calc(100dvh - 290px); }
.page-viewport .chapter-end { margin-top: 40px; color: var(--reader-muted); font-size: 13px; text-align: center; break-inside: avoid; }
.page-controls { display: flex; align-items: center; justify-content: center; gap: 18px; height: 58px; color: var(--reader-muted); font-size: 12px; }
.page-controls span { min-width: 58px; text-align: center; }
.page-controls :deep(.el-button) { border-color: var(--reader-border); color: var(--reader-text); background: var(--reader-surface); }

@media (max-width: 720px) {
  .reader { min-height: calc(100vh - 66px); padding: 12px 14px 60px; }
  .reader-toolbar { top: 76px; margin-bottom: 28px; }
  .reader-heading { margin-bottom: 38px; }
  .reader-content { padding: 30px 22px; border-radius: 15px; }
  .reader--paged { height: calc(100dvh - 66px); min-height: 460px; padding-bottom: 8px; }
  .reader--paged .reader-toolbar { top: 0; margin-bottom: 10px; }
  .page-viewport { height: calc(100dvh - 174px); min-height: 340px; padding: 28px 24px; border-radius: 15px; column-gap: 48px; }
  .paged-heading { margin-bottom: 32px; }
  .paged-heading h1 { font-size: 25px; }
  .page-controls { height: 50px; }
}
</style>

<style>
.reader-settings-popover .settings-panel { display: grid; gap: 10px; }
.reader-settings-popover .settings-title { display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px; }
.reader-settings-popover .settings-title strong { font-family: Georgia, "Noto Serif SC", serif; font-size: 17px; }
.reader-settings-popover label { display: flex; justify-content: space-between; color: #6f6b64; font-size: 13px; }
.reader-settings-popover label:not(:first-of-type) { margin-top: 5px; }
.reader-settings-popover label b { color: #9a8264; font-size: 12px; font-weight: 500; }
.reader-settings-popover .el-radio-group { width: 100%; }
.reader-settings-popover .el-radio-button { flex: 1; }
.reader-settings-popover .el-radio-button__inner { width: 100%; }
.reader-settings-popover .el-slider { padding: 0 6px; }
</style>
