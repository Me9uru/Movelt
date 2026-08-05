<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  ArrowLeft,
  Check,
  Collection,
  Reading,
  Search,
  Star,
  VideoPlay,
} from "@element-plus/icons-vue";
import {
  getNovelOverview,
  searchNovels,
  type NovelDetail,
  type NovelSummary,
  type Volume,
} from "./services/novel";
import type { ReaderDocument } from "./domain/reader";
import type { ReadingProgress } from "./domain/library";
import { bilinovelSource } from "./sources/bilinovel";
import NovelReader from "./components/reader/NovelReader.vue";
import { useLibrary } from "./composables/useLibrary";

type LibraryView = "search" | "bookshelf";
type View = LibraryView | "detail" | "reader";

const view = ref<View>("search");
const lastLibraryView = ref<LibraryView>("search");
const query = ref("");
const results = ref<NovelSummary[]>([]);
const detail = ref<NovelDetail | null>(null);
const catalogue = ref<Volume[]>([]);
const readerDocument = ref<ReaderDocument | null>(null);
const currentChapterId = ref<string | null>(null);
const loading = ref(false);
const bookshelfLoading = ref(true);
const errorMessage = ref("");
const {
  books,
  refreshBooks,
  addBook,
  removeBook,
  isOnBookshelf,
  progressFor,
  loadProgress,
  saveProgress,
} = useLibrary();

const chapterCount = computed(() =>
  catalogue.value.reduce((total, volume) => total + volume.chapters.length, 0),
);
const onBookshelf = computed(() => detail.value ? isOnBookshelf(detail.value) : false);
const currentProgress = computed(() => detail.value ? progressFor(detail.value) : null);
const readerInitialProgress = computed(() => {
  const progress = currentProgress.value;
  return progress?.documentId === currentChapterId.value ? progress : null;
});

function describeError(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object") {
    const value = error as { message?: unknown; code?: unknown };
    if (typeof value.message === "string") return value.message;
    if (typeof value.code === "string") return value.code;
  }
  return "请求失败，请稍后重试";
}

async function run<T>(task: () => Promise<T>): Promise<T | null> {
  loading.value = true;
  errorMessage.value = "";
  try {
    return await task();
  } catch (error) {
    errorMessage.value = describeError(error);
    return null;
  } finally {
    loading.value = false;
  }
}

async function search() {
  if (!query.value.trim()) return;
  const response = await run(() => searchNovels(query.value.trim()));
  if (response) {
    results.value = response.items;
    view.value = "search";
  }
}

function openLibraryView(nextView: LibraryView) {
  view.value = nextView;
  lastLibraryView.value = nextView;
  errorMessage.value = "";
  window.scrollTo({ top: 0, behavior: "smooth" });
}

async function openNovel(novel: NovelSummary) {
  if (view.value === "search" || view.value === "bookshelf") {
    lastLibraryView.value = view.value;
  }
  const response = await run(async () => {
    const overview = await getNovelOverview(novel.id);
    await loadProgress(overview.detail);
    return overview;
  });
  if (response) {
    detail.value = response.detail;
    catalogue.value = response.volumes;
    view.value = "detail";
    window.scrollTo({ top: 0, behavior: "smooth" });
  }
}

async function openChapter(chapterId: string) {
  if (!detail.value) return;
  const response = await run(async () => {
    const book = detail.value!;
    const document = await bilinovelSource.loadDocument(book.id, chapterId);
    const existing = progressFor(book);
    await saveProgress(book, {
      documentId: chapterId,
      documentTitle: document.title,
      location: existing?.documentId === chapterId ? existing.location : 0,
    });
    return document;
  });
  if (response) {
    readerDocument.value = response;
    currentChapterId.value = chapterId;
    view.value = "reader";
  }
}

async function toggleBookshelf() {
  if (!detail.value) return;
  const book = detail.value;
  await run(() => isOnBookshelf(book) ? removeBook(book) : addBook(book));
}

function continueReading() {
  if (currentProgress.value) void openChapter(currentProgress.value.documentId);
}

function recordProgress(location: number) {
  if (!detail.value || !readerDocument.value || !currentChapterId.value) return;
  void saveProgress(detail.value, {
    documentId: currentChapterId.value,
    documentTitle: readerDocument.value.title,
    location,
  }).catch((error) => {
    errorMessage.value = describeError(error);
  });
}

function progressPercent(progress: ReadingProgress | null): number {
  return progress ? Math.round(progress.location * 100) : 0;
}

function back() {
  errorMessage.value = "";
  if (view.value === "reader") {
    view.value = "detail";
  } else {
    view.value = lastLibraryView.value;
  }
  window.scrollTo({ top: 0, behavior: "smooth" });
}

onMounted(async () => {
  try {
    await refreshBooks();
  } catch (error) {
    errorMessage.value = describeError(error);
  } finally {
    bookshelfLoading.value = false;
  }
});
</script>

<template>
  <div class="page-bg">
    <header v-if="view === 'detail' || view === 'reader'" class="topbar">
      <div class="topbar-inner detail-topbar">
        <el-button
          class="back-button"
          :icon="ArrowLeft"
          round
          @click="back"
        >
          {{ view === "reader" ? "返回目录" : `返回${lastLibraryView === "bookshelf" ? "书架" : "小说"}` }}
        </el-button>
      </div>
    </header>

    <main class="app-shell">
      <el-alert
        v-if="errorMessage"
        class="error-alert"
        :title="errorMessage"
        type="error"
        show-icon
        closable
        @close="errorMessage = ''"
      />

      <section v-if="view === 'search'" class="search-view">
        <div class="hero">
          <form class="search-box" @submit.prevent="search">
            <el-input
              v-model="query"
              :prefix-icon="Search"
              size="large"
              clearable
              aria-label="小说名或作者"
              placeholder="输入小说名或作者"
            />
            <el-button
              native-type="submit"
              type="primary"
              size="large"
              :loading="loading"
              :disabled="!query.trim()"
            >
              搜索作品
            </el-button>
          </form>
        </div>

        <section class="library-section">
          <div v-if="results.length" class="section-heading">
            <div>
              <span>SEARCH RESULTS</span>
              <h2>找到的作品</h2>
            </div>
            <el-tag round effect="plain">{{ results.length }} 本</el-tag>
          </div>

          <div v-if="loading" class="result-grid" aria-label="正在搜索">
            <el-card v-for="item in 6" :key="item" shadow="never" class="book-card skeleton-card">
              <el-skeleton animated>
                <template #template>
                  <el-skeleton-item variant="image" class="skeleton-cover" />
                  <el-skeleton-item variant="h3" style="width: 82%" />
                  <el-skeleton-item variant="text" style="width: 48%" />
                </template>
              </el-skeleton>
            </el-card>
          </div>

          <el-empty
            v-else-if="results.length === 0"
            :image-size="112"
            description="输入书名或作者开始搜索"
          >
            <p class="empty-tip">首次加载书库索引可能需要十几秒，请稍候。</p>
          </el-empty>

          <div v-else class="result-grid">
            <el-card
              v-for="novel in results"
              :key="novel.id"
              class="book-card"
              shadow="hover"
              tabindex="0"
              @click="openNovel(novel)"
              @keydown.enter="openNovel(novel)"
            >
              <el-image
                v-if="novel.cover_url"
                class="book-cover"
                :src="novel.cover_url"
                :alt="novel.title"
                fit="cover"
                lazy
              >
                <template #error>
                  <div class="cover-placeholder"><el-icon><Collection /></el-icon></div>
                </template>
              </el-image>
              <div v-else class="cover-placeholder"><el-icon><Collection /></el-icon></div>
              <div class="book-meta">
                <strong>{{ novel.title }}</strong>
                <span>查看详情 <span aria-hidden="true">→</span></span>
              </div>
            </el-card>
          </div>
        </section>
      </section>

      <section v-else-if="view === 'bookshelf'" class="bookshelf-view">
        <section v-loading="bookshelfLoading" class="bookshelf-section">
          <div class="section-heading">
            <div>
              <span>MY BOOKSHELF</span>
              <h2>我的书架</h2>
            </div>
            <el-tag round effect="plain">{{ books.length }} 本</el-tag>
          </div>

          <div class="result-grid">
            <el-card
              v-for="entry in books"
              :key="`${entry.book.source}:${entry.book.id}`"
              class="book-card shelf-card"
              shadow="hover"
              tabindex="0"
              @click="openNovel(entry.book)"
              @keydown.enter="openNovel(entry.book)"
            >
              <el-image
                v-if="entry.book.cover_url"
                class="book-cover"
                :src="entry.book.cover_url"
                :alt="entry.book.title"
                fit="cover"
                lazy
              >
                <template #error>
                  <div class="cover-placeholder"><el-icon><Collection /></el-icon></div>
                </template>
              </el-image>
              <div v-else class="cover-placeholder"><el-icon><Collection /></el-icon></div>
              <div class="book-meta">
                <strong>{{ entry.book.title }}</strong>
                <template v-if="progressFor(entry.book)">
                  <span class="shelf-progress-title">{{ progressFor(entry.book)?.documentTitle }}</span>
                  <el-progress
                    :percentage="progressPercent(progressFor(entry.book))"
                    :stroke-width="4"
                    :show-text="false"
                  />
                </template>
                <span v-else>尚未开始阅读</span>
              </div>
            </el-card>
          </div>

          <el-empty
            v-if="!bookshelfLoading && books.length === 0"
            :image-size="112"
            description="书架还是空的"
          >
            <el-button type="primary" plain @click="openLibraryView('search')">
              去找小说
            </el-button>
          </el-empty>
        </section>
      </section>

      <section v-else-if="view === 'detail' && detail" v-loading="loading" class="detail-view">
        <el-card class="book-profile" shadow="never">
          <div class="book-heading">
            <el-image
              v-if="detail.cover_url"
              class="detail-cover"
              :src="detail.cover_url"
              :alt="detail.title"
              fit="cover"
            />
            <div v-else class="detail-cover cover-placeholder"><el-icon><Collection /></el-icon></div>

            <div class="detail-copy">
              <div class="detail-tags">
                <el-tag round effect="light">{{ detail.source }}</el-tag>
                <el-tag v-if="detail.status" round effect="plain" type="success">{{ detail.status }}</el-tag>
              </div>
              <h1>{{ detail.title }}</h1>
              <p class="author">{{ detail.author || "佚名" }}</p>
              <div class="stats">
                <span><strong>{{ catalogue.length }}</strong> 卷</span>
                <el-divider direction="vertical" />
                <span><strong>{{ chapterCount }}</strong> 章</span>
                <template v-if="detail.updated_at">
                  <el-divider direction="vertical" />
                  <span>更新于 {{ detail.updated_at }}</span>
                </template>
              </div>
              <p class="description">{{ detail.description || "暂无作品简介。" }}</p>
              <div class="detail-actions">
                <el-button
                  :type="onBookshelf ? 'default' : 'primary'"
                  :icon="onBookshelf ? Check : Star"
                  size="large"
                  @click="toggleBookshelf"
                >
                  {{ onBookshelf ? "已加入书架" : "加入书架" }}
                </el-button>
                <el-button
                  v-if="currentProgress"
                  :icon="VideoPlay"
                  size="large"
                  @click="continueReading"
                >
                  继续阅读 · {{ currentProgress.documentTitle }}
                </el-button>
              </div>
            </div>
          </div>
        </el-card>

        <div class="catalogue-heading">
          <div>
            <span>CATALOGUE</span>
            <h2>作品目录</h2>
          </div>
          <p>共 {{ catalogue.length }} 卷 · {{ chapterCount }} 章</p>
        </div>

        <el-collapse class="catalogue" :model-value="catalogue[0] ? [0] : []">
          <el-collapse-item v-for="(volume, volumeIndex) in catalogue" :key="`${volume.title}-${volumeIndex}`" :name="volumeIndex">
            <template #title>
              <div class="volume-title">
                <span class="volume-index">{{ String(volumeIndex + 1).padStart(2, "0") }}</span>
                <strong>{{ volume.title }}</strong>
                <el-tag size="small" round effect="plain">{{ volume.chapters.length }} 章</el-tag>
              </div>
            </template>
            <div class="chapter-list">
              <button
                v-for="(item, chapterIndex) in volume.chapters"
                :key="item.id"
                type="button"
                :class="{ 'chapter-current': currentProgress?.documentId === item.id }"
                @click="openChapter(item.id)"
              >
                <span class="chapter-number">{{ String(chapterIndex + 1).padStart(2, "0") }}</span>
                <span>
                  {{ item.title }}
                  <small v-if="currentProgress?.documentId === item.id">
                    上次读到 {{ progressPercent(currentProgress) }}%
                  </small>
                </span>
                <el-icon><Reading /></el-icon>
              </button>
            </div>
          </el-collapse-item>
        </el-collapse>
      </section>

      <NovelReader
        v-else-if="view === 'reader' && readerDocument"
        :document="readerDocument"
        :loading="loading"
        :initial-progress="readerInitialProgress"
        @back="back"
        @progress="recordProgress"
      />
    </main>

    <nav
      v-if="view === 'search' || view === 'bookshelf'"
      class="view-dock"
      aria-label="主栏目"
    >
      <button
        type="button"
        :class="{ active: view === 'search' }"
        :aria-current="view === 'search' ? 'page' : undefined"
        @click="openLibraryView('search')"
      >
        <el-icon><Search /></el-icon>
        <span>小说</span>
      </button>
      <button
        type="button"
        :class="{ active: view === 'bookshelf' }"
        :aria-current="view === 'bookshelf' ? 'page' : undefined"
        @click="openLibraryView('bookshelf')"
      >
        <el-icon><Collection /></el-icon>
        <span>书架</span>
        <small v-if="books.length">{{ books.length }}</small>
      </button>
    </nav>
  </div>
</template>

<style>
:root {
  font-family: Inter, "Noto Sans SC", "Microsoft YaHei", sans-serif;
  color: #24231f;
  background: #f7f6f2;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  --el-color-primary: #896f4c;
  --el-color-primary-light-3: #a9967b;
  --el-color-primary-light-5: #c4b7a6;
  --el-color-primary-light-7: #ddd5cb;
  --el-color-primary-light-8: #e9e4de;
  --el-color-primary-light-9: #f4f1ed;
  --el-color-primary-dark-2: #6e593d;
  --el-border-radius-base: 10px;
  --el-border-color: #e5e1d9;
}

* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body { margin: 0; min-width: 320px; min-height: 100vh; }
button, input { font: inherit; }
button { cursor: pointer; }
.page-bg { min-height: 100vh; background: radial-gradient(circle at 50% -10%, #fff 0, #faf9f6 36%, #f4f2ed 100%); }

.topbar { position: sticky; top: 0; z-index: 20; border-bottom: 1px solid rgb(218 214 205 / 72%); background: rgb(250 249 246 / 86%); backdrop-filter: blur(18px); }
.topbar-inner { width: min(1160px, calc(100% - 48px)); height: 64px; margin: auto; display: flex; align-items: center; }
.detail-topbar { justify-content: flex-start; }
.view-dock { position: fixed; bottom: max(20px, env(safe-area-inset-bottom)); left: 50%; z-index: 30; display: flex; align-items: center; gap: 5px; padding: 5px; border: 1px solid rgb(218 214 205 / 88%); border-radius: 16px; background: rgb(255 255 255 / 88%); box-shadow: 0 14px 40px rgb(50 45 38 / 16%); backdrop-filter: blur(18px); transform: translateX(-50%); }
.view-dock button { position: relative; display: flex; align-items: center; gap: 7px; min-width: 92px; height: 38px; padding: 0 14px; border: 0; border-radius: 10px; color: #817b71; background: transparent; justify-content: center; transition: color .2s ease, background .2s ease, box-shadow .2s ease; }
.view-dock button:hover { color: #4e493f; background: #f3f0ea; }
.view-dock button.active { color: #fff; background: #2f302b; box-shadow: 0 7px 16px rgb(47 48 43 / 18%); }
.view-dock .el-icon { font-size: 17px; }
.view-dock small { display: grid; place-items: center; min-width: 18px; height: 18px; padding: 0 5px; border-radius: 9px; color: #766651; background: #eee8df; font-size: 10px; }
.view-dock button.active small { color: #3e3931; background: #fff; }
.back-button { background: rgb(255 255 255 / 70%); }

.app-shell { width: min(1160px, calc(100% - 48px)); margin: 0 auto; padding: 0 0 112px; }
.app-shell:has(.reader--paged) { padding-bottom: 0; }
.error-alert { position: sticky; top: 88px; z-index: 15; margin: 16px auto -8px; max-width: 760px; box-shadow: 0 10px 30px rgb(100 45 38 / 10%); }
.hero { padding: 56px 0 60px; text-align: center; }
.hero .eyebrow { margin-bottom: 20px; letter-spacing: .12em; }
.hero h1 { margin: 0; font-family: Georgia, "Noto Serif SC", serif; font-size: clamp(36px, 5vw, 58px); font-weight: 500; letter-spacing: -.035em; line-height: 1.25; }
.hero > p { margin: 20px 0 34px; color: #817d74; font-size: 16px; }
.search-box { display: flex; gap: 10px; width: min(680px, 100%); margin: 0 auto; padding: 7px; border: 1px solid #e2ded6; border-radius: 16px; background: rgb(255 255 255 / 88%); box-shadow: 0 18px 50px rgb(70 61 48 / 10%); }
.search-box .el-input__wrapper { box-shadow: none; background: transparent; }
.search-box .el-button { min-width: 118px; height: 44px; border-radius: 10px; }

.bookshelf-view { padding-top: 56px; }
.bookshelf-section { min-height: 420px; margin-bottom: 70px; }
.library-section { min-height: 320px; }
.section-heading, .catalogue-heading { display: flex; align-items: end; justify-content: space-between; margin-bottom: 26px; }
.section-heading span, .catalogue-heading > div > span { color: #a28c70; font-size: 11px; font-weight: 700; letter-spacing: .18em; }
.section-heading h2, .catalogue-heading h2 { margin: 5px 0 0; font-family: Georgia, "Noto Serif SC", serif; font-size: 28px; font-weight: 500; }
.result-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(165px, 1fr)); gap: 24px; }
.book-card { overflow: hidden; border: 1px solid #e8e4dc; border-radius: 14px; background: rgb(255 255 255 / 72%); cursor: pointer; transition: transform .25s ease, box-shadow .25s ease; }
.book-card:hover { transform: translateY(-5px); }
.book-card .el-card__body { padding: 10px 10px 16px; }
.book-cover, .cover-placeholder { display: block; width: 100%; aspect-ratio: 3 / 4; border-radius: 9px; overflow: hidden; background: linear-gradient(145deg, #ece8df, #ddd6ca); }
.cover-placeholder { display: grid; place-items: center; color: #9d9486; }
.cover-placeholder .el-icon { font-size: 36px; }
.book-meta { padding: 14px 5px 0; }
.book-meta strong { display: -webkit-box; overflow: hidden; min-height: 44px; color: #34332e; font-size: 14px; line-height: 1.55; -webkit-box-orient: vertical; -webkit-line-clamp: 2; }
.book-meta > span { display: block; margin-top: 8px; color: #a18a6d; font-size: 12px; }
.shelf-card .book-meta strong { min-height: auto; }
.shelf-progress-title { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.shelf-card .el-progress { margin-top: 9px; }
.skeleton-card { cursor: default; }
.skeleton-card:hover { transform: none; }
.skeleton-cover { width: 100%; height: auto; aspect-ratio: 3 / 4; margin-bottom: 16px; border-radius: 9px; }
.empty-tip { margin: -8px 0 0; color: #aaa49a; font-size: 12px; }

.detail-view { padding-top: 48px; }
.book-profile { border: 1px solid #e6e1d8; border-radius: 22px; background: rgb(255 255 255 / 72%); box-shadow: 0 24px 70px rgb(65 58 47 / 8%); }
.book-profile .el-card__body { padding: 40px; }
.book-heading { display: grid; grid-template-columns: 210px minmax(0, 1fr); gap: 44px; align-items: start; }
.detail-cover { width: 100%; aspect-ratio: 3 / 4; border-radius: 12px; overflow: hidden; box-shadow: 0 18px 36px rgb(52 47 39 / 20%); }
.detail-tags { display: flex; gap: 8px; }
.detail-copy h1 { margin: 18px 0 8px; font-family: Georgia, "Noto Serif SC", serif; font-size: clamp(30px, 4vw, 42px); font-weight: 500; line-height: 1.25; }
.author { margin: 0; color: #80796e; font-size: 15px; }
.stats { display: flex; align-items: center; flex-wrap: wrap; gap: 5px; margin: 24px 0; color: #969086; font-size: 13px; }
.stats strong { color: #5f584d; font-size: 16px; }
.description { display: -webkit-box; overflow: hidden; margin: 0; color: #666157; line-height: 1.9; white-space: pre-line; -webkit-box-orient: vertical; -webkit-line-clamp: 6; }
.detail-actions { display: flex; flex-wrap: wrap; gap: 10px; margin-top: 28px; }
.detail-actions .el-button + .el-button { margin-left: 0; }
.catalogue-heading { margin-top: 58px; }
.catalogue-heading > p { margin: 0; color: #979188; font-size: 13px; }
.catalogue { overflow: hidden; border: 1px solid #e6e1d8; border-radius: 16px; background: rgb(255 255 255 / 68%); }
.catalogue .el-collapse-item__header { height: 66px; padding: 0 24px; border-color: #ebe7e0; background: transparent; }
.catalogue .el-collapse-item__wrap { border-color: #ebe7e0; background: transparent; }
.catalogue .el-collapse-item__content { padding: 0; }
.volume-title { display: flex; align-items: center; min-width: 0; width: 100%; gap: 14px; }
.volume-title strong { overflow: hidden; margin-right: auto; text-overflow: ellipsis; white-space: nowrap; }
.volume-index { color: #b09a7e; font-family: Georgia, serif; }
.chapter-list { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); padding: 10px 18px 18px; gap: 6px 18px; }
.chapter-list button { display: grid; grid-template-columns: 28px minmax(0, 1fr) 20px; align-items: center; gap: 8px; width: 100%; padding: 12px 10px; border: 0; border-radius: 9px; color: #5a564e; background: transparent; text-align: left; transition: color .2s, background .2s; }
.chapter-list button:hover { color: var(--el-color-primary); background: var(--el-color-primary-light-9); }
.chapter-list button > span:nth-child(2) { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.chapter-list button > span:nth-child(2) small { display: block; margin-top: 3px; color: #a18a6d; font-size: 11px; }
.chapter-list button.chapter-current { color: var(--el-color-primary); background: var(--el-color-primary-light-9); }
.chapter-list button.chapter-current .el-icon { opacity: 1; }
.chapter-number { color: #b2aca2; font-family: Georgia, serif; font-size: 12px; }
.chapter-list .el-icon { opacity: 0; transition: opacity .2s; }
.chapter-list button:hover .el-icon { opacity: 1; }

@media (max-width: 720px) {
  .topbar-inner, .app-shell { width: min(100% - 28px, 1160px); }
  .topbar-inner { height: 58px; }
  .view-dock { padding: 4px; }
  .view-dock button { min-width: 76px; height: 36px; padding: 0 10px; }
  .view-dock small { display: none; }
  .hero { padding: 64px 0 50px; }
  .hero h1 { font-size: 36px; }
  .hero > p { padding: 0 18px; line-height: 1.7; }
  .search-box { gap: 6px; }
  .search-box .el-button { min-width: 92px; padding: 8px 14px; }
  .result-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
  .book-heading { grid-template-columns: 1fr; gap: 24px; }
  .detail-cover { width: 140px; margin: 0 auto; }
  .detail-copy { text-align: center; }
  .detail-tags, .stats { justify-content: center; }
  .detail-actions { justify-content: center; }
  .book-profile .el-card__body { padding: 20px; }
  .detail-copy h1 { margin-top: 12px; font-size: 25px; }
  .stats { margin: 15px 0; }
  .description { font-size: 14px; text-align: left; -webkit-line-clamp: 5; }
  .chapter-list { grid-template-columns: 1fr; padding: 8px 10px 14px; }
  .catalogue .el-collapse-item__header { padding: 0 14px; }
}
</style>
