<script setup lang="ts">
import { ArrowLeft, ArrowRight } from "@element-plus/icons-vue";
import { ElMessage } from "element-plus";
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { getMangaChapterPages, getManga, getMangaPageBatch, saveMangaReadPosition, type MangaDetail } from "../../services/manga";
import { useReaderSettings } from "../../composables/useReaderSettings";
import { getErrorMessage, showError } from "../../utils/error";
import { clearWebviewCache } from "../../services/settings";
import ErrorState from "../../components/common/ErrorState.vue";

const route = useRoute(); const router = useRouter();
const mangaId = computed(() => String(route.params.mangaId)); const chapterId = computed(() => String(route.params.chapterId));
const { settings } = useReaderSettings("manga");
const manga = ref<MangaDetail | null>(null); const pages = ref<string[]>([]); const loading = ref(true); const error = ref(""); const settingsVisible = ref(false); const pageList = ref<HTMLElement | null>(null); const currentPage = ref(0); const requestedBatches = new Set<number>(); let loadObserver: IntersectionObserver | undefined; let progressObserver: IntersectionObserver | undefined; let savedPage = -1; let saveTimer: number | undefined;
async function loadPage(index: number): Promise<void> { if (pages.value[index]) return; const batchStart = Math.floor(index / 12) * 12; if (requestedBatches.has(batchStart)) return; requestedBatches.add(batchStart); try { const batch = await getMangaPageBatch(chapterId.value, index); batch.pageUrls.forEach((url, offset) => { if (batch.startIndex + offset < pages.value.length) pages.value[batch.startIndex + offset] = url; }); } catch (value) { error.value = getErrorMessage(value, "无法加载漫画页面"); showError(value, "无法加载漫画页面"); } finally { requestedBatches.delete(batchStart); } }
async function load(): Promise<void> { loading.value = true; error.value = ""; requestedBatches.clear(); if (saveTimer !== undefined) window.clearTimeout(saveTimer); saveTimer = undefined; savedPage = -1; currentPage.value = 0; const pagesRequest = getMangaChapterPages(mangaId.value, chapterId.value); const detailRequest = getManga(mangaId.value); try { const [list, detail] = await Promise.all([pagesRequest, detailRequest]); manga.value = detail; pages.value = Array.from({ length: list.pageCount }, (_, index) => list.firstPageUrls[index] || ""); const position = list.readPosition ?? detail.readPosition; const page = position?.chapterId === chapterId.value ? Number.parseInt(position.position, 10) : 1; savedPage = Math.min(Math.max(Number.isFinite(page) ? page - 1 : 0, 0), Math.max(0, pages.value.length - 1)); currentPage.value = savedPage; await nextTick(); await loadPage(currentPage.value); await nextTick(); observePages(); if (settings.mode === "scroll") pageList.value?.querySelector<HTMLElement>(`[data-page-index="${currentPage.value}"]`)?.scrollIntoView({ block: "start", behavior: "auto" }); } catch (value) { error.value = getErrorMessage(value, "无法加载漫画页面"); showError(value, "无法加载漫画页面"); } finally { loading.value = false; } }
function saveProgress(immediate = false): void { if (!manga.value || savedPage < 0) return; if (saveTimer !== undefined) window.clearTimeout(saveTimer); const save = () => { saveTimer = undefined; void saveMangaReadPosition(mangaId.value, chapterId.value, savedPage + 1).catch((errorValue) => { showError(errorValue, "保存阅读进度失败"); }); }; if (immediate) save(); else saveTimer = window.setTimeout(save, 400); }
function setCurrentPage(index: number): void { if (index < 0 || index >= pages.value.length || index === currentPage.value) return; currentPage.value = index; savedPage = index; saveProgress(); }
function observePages(): void { loadObserver?.disconnect(); progressObserver?.disconnect(); if (settings.mode !== "scroll") return; loadObserver = new IntersectionObserver((entries) => { for (const entry of entries) { if (!entry.isIntersecting) continue; const index = Number((entry.target as HTMLElement).dataset.pageIndex); if (Number.isInteger(index)) void loadPage(index); } }, { rootMargin: "800px 0px", threshold: 0.01 }); progressObserver = new IntersectionObserver((entries) => { const visible = entries.filter((entry) => entry.isIntersecting).map((entry) => Number((entry.target as HTMLElement).dataset.pageIndex)).filter(Number.isInteger); if (visible.length) setCurrentPage(Math.min(...visible)); }, { rootMargin: "0px 0px -50%", threshold: 0.01 }); pageList.value?.querySelectorAll<HTMLElement>(".manga-reader-page").forEach((page) => { loadObserver?.observe(page); progressObserver?.observe(page); }); }
function chapterOffset(offset: number): void { const chapters = manga.value?.chapters ?? []; const index = chapters.findIndex((chapter) => chapter.id === chapterId.value); const next = chapters[index + offset]; if (next) { savedPage = currentPage.value; saveProgress(true); void router.replace({ name: "manga-reader", params: { mangaId: mangaId.value, chapterId: next.id } }); } }
function changePage(offset: number): void { const next = currentPage.value + offset; if (next < 0 || next >= pages.value.length) { chapterOffset(offset); return; } currentPage.value = next; savedPage = next; saveProgress(); void loadPage(next); }
function pageOffsetForSide(side: "left" | "right"): number { const leftOffset = settings.pageTurnDirection === "left-next" ? 1 : -1; return side === "left" ? leftOffset : -leftOffset; }
function handleReaderClick(event: MouseEvent): void { if ((event.target as HTMLElement).closest("button, a")) return; if (settings.mode === "paged") { const start = window.innerWidth / 3; const end = start * 2; if (event.clientX < start) { changePage(pageOffsetForSide("left")); return; } if (event.clientX > end) { changePage(pageOffsetForSide("right")); return; } } settingsVisible.value = true; }
async function clearBrowsingData(): Promise<void> { try { await clearWebviewCache(); ElMessage.success("已清除 WebView 浏览数据"); } catch { ElMessage.error("清除 WebView 数据失败"); } }
onMounted(() => void load()); onBeforeUnmount(() => { loadObserver?.disconnect(); progressObserver?.disconnect(); savedPage = currentPage.value; saveProgress(true); });
watch(chapterId, () => void load());
watch(() => settings.mode, () => { currentPage.value = Math.max(0, savedPage); void nextTick(() => { observePages(); if (settings.mode === "paged") void loadPage(currentPage.value); }); });
</script>
<template>
  <article class="manga-reader" :class="[`manga-reader--${settings.theme}`, `manga-reader--${settings.mode}`]" v-loading="loading" @click="handleReaderClick">
    <Teleport to="body">
      <Transition name="manga-reader-nav">
        <nav v-if="settingsVisible" class="manga-reader-nav" :class="`manga-reader-nav--${settings.theme}`" aria-label="漫画章节导航">
          <el-button circle :icon="ArrowLeft" :disabled="!manga || manga.chapters.findIndex((chapter) => chapter.id === chapterId) <= 0" aria-label="上一话" title="上一话" @click="chapterOffset(-1)" />
          <button class="manga-reader-title" type="button" :title="manga?.title" @click="router.push({ name: 'manga-detail', params: { mangaId } })">{{ manga?.title || "漫画阅读" }}</button>
          <el-button circle :icon="ArrowRight" :disabled="!manga || manga.chapters.findIndex((chapter) => chapter.id === chapterId) >= manga.chapters.length - 1" aria-label="下一话" title="下一话" @click="chapterOffset(1)" />
        </nav>
      </Transition>
    </Teleport>
    <ErrorState v-if="error" title="漫画页面加载失败" :message="error" :loading="loading" @retry="load" />
    <section v-else ref="pageList" class="manga-reader-pages">
      <p v-if="!loading && pages.length === 0" class="manga-reader-pending">正在获取章节页码…</p>
      <template v-if="settings.mode === 'scroll'">
        <template v-for="(page, index) in pages" :key="index">
          <img v-if="page" class="manga-reader-page" :data-page-index="index" :src="page" :alt="`第 ${index + 1} 页`" />
          <div v-else class="manga-reader-page manga-reader-page-placeholder" :data-page-index="index">加载第 {{ index + 1 }} 页…</div>
        </template>
      </template>
      <template v-else-if="pages.length">
        <img v-if="pages[currentPage]" class="manga-reader-page" :data-page-index="currentPage" :src="pages[currentPage]" :alt="`第 ${currentPage + 1} 页`" />
        <div v-else class="manga-reader-page manga-reader-page-placeholder" :data-page-index="currentPage">加载第 {{ currentPage + 1 }} 页…</div>
      </template>
    </section>
    <el-drawer v-model="settingsVisible" class="reader-settings-drawer" direction="btt" size="min(360px, calc(100dvh - env(safe-area-inset-top) - 138px))" :with-header="false" :z-index="2000" append-to-body>
      <div class="settings-panel">
        <div class="settings-handle" aria-hidden="true"></div>
        <div class="settings-title"><strong>阅读设置</strong></div>
        <label>阅读模式</label>
        <el-radio-group v-model="settings.mode" size="small"><el-radio-button value="scroll">滚动阅读</el-radio-button><el-radio-button value="paged">分页阅读</el-radio-button></el-radio-group>
        <template v-if="settings.mode === 'paged'">
          <label>翻页点击方向</label>
          <el-radio-group v-model="settings.pageTurnDirection" size="small"><el-radio-button value="left-previous">左边上一页 · 右边下一页</el-radio-button><el-radio-button value="left-next">左边下一页 · 右边上一页</el-radio-button></el-radio-group>
        </template>
        <label>背景主题</label>
        <el-radio-group v-model="settings.theme" size="small"><el-radio-button value="paper">纸张</el-radio-button><el-radio-button value="light">明亮</el-radio-button><el-radio-button value="night">夜间</el-radio-button></el-radio-group>
        <label>存储</label>
        <el-button size="small" @click="clearBrowsingData">清除 WebView 数据</el-button>
      </div>
    </el-drawer>
  </article>
</template>
