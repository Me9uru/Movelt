<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import {
  ArrowLeft,
  Collection,
  Search,
} from "@element-plus/icons-vue";
import {
  getNovelOverview,
  listNovelSources,
  searchNovels,
  type NovelDetail,
  type NovelSourceInfo,
  type NovelSummary,
  type Volume,
} from "./services/novel";
import type { ReaderDocument } from "./domain/reader";
import { networkNovelSource } from "./sources/networkNovel";
import BookshelfView from "./components/library/BookshelfView.vue";
import LoadingOverlay from "./components/common/LoadingOverlay.vue";
import NovelDetailView from "./components/library/NovelDetailView.vue";
import NovelSearchView from "./components/library/NovelSearchView.vue";
import NovelReader from "./components/reader/NovelReader.vue";
import { useLibrary } from "./composables/useLibrary";
import { useReaderSettings } from "./composables/useReaderSettings";

type LibraryView = "search" | "bookshelf";
type View = LibraryView | "detail" | "reader";
type LoadingAction = "search" | "novel" | "chapter" | "bookshelf";

const historyViewKey = "movelView";

const view = ref<View>("search");
const lastLibraryView = ref<LibraryView>("search");
const query = ref("");
const sourceOptions = ref<NovelSourceInfo[]>([]);
const selectedSource = ref("");
const results = ref<NovelSummary[]>([]);
const detail = ref<NovelDetail | null>(null);
const catalogue = ref<Volume[]>([]);
const readerDocument = ref<ReaderDocument | null>(null);
const currentChapterId = ref<string | null>(null);
const loading = ref(false);
const loadingAction = ref<LoadingAction | null>(null);
const bookshelfLoading = ref(true);
const errorMessage = ref("");
const { settings: readerSettings } = useReaderSettings();
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

const chapterIds = computed(() =>
  catalogue.value.flatMap((volume) => volume.chapters.map((chapter) => chapter.id)),
);
const nextChapterId = computed(() => {
  if (!currentChapterId.value) return null;
  const currentIndex = chapterIds.value.indexOf(currentChapterId.value);
  return currentIndex >= 0 ? chapterIds.value[currentIndex + 1] ?? null : null;
});
const onBookshelf = computed(() => detail.value ? isOnBookshelf(detail.value) : false);
const currentProgress = computed(() => detail.value ? progressFor(detail.value) : null);
const readerInitialProgress = computed(() => {
  const progress = currentProgress.value;
  return progress?.documentId === currentChapterId.value ? progress : null;
});
const loadingCopy = computed(() => {
  switch (loadingAction.value) {
    case "search":
      return { title: "正在搜索作品", hint: "首次加载书库索引可能需要十几秒" };
    case "novel":
      return { title: "正在加载作品详情", hint: "正在获取简介与章节目录" };
    case "chapter":
      return { title: "正在加载章节", hint: "内容较多时可能需要稍候" };
    case "bookshelf":
      return { title: "正在更新书架", hint: "请稍候" };
    default:
      return { title: "正在加载", hint: "请稍候" };
  }
});
const showLoadingOverlay = computed(() =>
  loading.value || (view.value === "bookshelf" && bookshelfLoading.value),
);
const loadingLabel = computed(() =>
  bookshelfLoading.value && view.value === "bookshelf"
    ? "正在加载书架"
    : loadingCopy.value.title,
);

const stopThemeSync = watch(
  () => readerSettings.theme,
  (theme) => {
    document.documentElement.dataset.readerTheme = theme;
  },
  { immediate: true },
);

function describeError(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object") {
    const value = error as { message?: unknown; code?: unknown };
    if (typeof value.message === "string") return value.message;
    if (typeof value.code === "string") return value.code;
  }
  return "请求失败，请稍后重试";
}

let activeLoadSeq = 0;

async function run<T>(action: LoadingAction, task: () => Promise<T>): Promise<T | null> {
  if (loading.value) return null;
  const seq = ++activeLoadSeq;
  loading.value = true;
  loadingAction.value = action;
  errorMessage.value = "";
  try {
    const result = await task();
    if (seq !== activeLoadSeq) return null;
    return result;
  } catch (error) {
    if (seq !== activeLoadSeq) return null;
    errorMessage.value = describeError(error);
    return null;
  } finally {
    if (seq === activeLoadSeq) {
      loading.value = false;
      loadingAction.value = null;
    }
  }
}

async function search() {
  if (!query.value.trim()) return;
  const searchQuery = query.value.trim();
  const response = await run("search", () => searchNovels(selectedSource.value, searchQuery));
  if (response) {
    results.value = response.items;
    view.value = "search";
  }
}

function openLibraryView(nextView: LibraryView) {
  view.value = nextView;
  lastLibraryView.value = nextView;
  replaceHistoryView(nextView);
  errorMessage.value = "";
  window.scrollTo({ top: 0 });
}

function historyState(nextView: View): Record<string, unknown> {
  const currentState = window.history.state;
  const state = currentState && typeof currentState === "object"
    ? currentState as Record<string, unknown>
    : {};
  return { ...state, [historyViewKey]: nextView };
}

function replaceHistoryView(nextView: View) {
  window.history.replaceState(historyState(nextView), "");
}

function enterHistoryView(nextView: "detail" | "reader") {
  view.value = nextView;
  window.history.pushState(historyState(nextView), "");
  window.scrollTo({ top: 0 });
}

function handleHistoryBack(event: PopStateEvent) {
  const nextView = event.state?.[historyViewKey];
  if (nextView === "reader" && readerDocument.value) {
    view.value = "reader";
  } else if (nextView === "detail" && detail.value) {
    view.value = "detail";
  } else if (nextView === "bookshelf" || nextView === "search") {
    view.value = nextView;
    lastLibraryView.value = nextView;
  } else {
    view.value = lastLibraryView.value;
    replaceHistoryView(lastLibraryView.value);
  }
  errorMessage.value = "";
  window.scrollTo({ top: 0 });
}

function handleAndroidBack(event: Event) {
  if (view.value === "detail" || view.value === "reader") {
    event.preventDefault();
    back();
  }
}

async function openNovel(novel: NovelSummary) {
  if (view.value === "search" || view.value === "bookshelf") {
    lastLibraryView.value = view.value;
  }
  const response = await run("novel", async () => {
    const overview = await getNovelOverview(novel.source, novel.id);
    await loadProgress(overview.detail);
    return overview;
  });
  if (response) {
    detail.value = response.detail;
    catalogue.value = response.volumes;
    enterHistoryView("detail");
  }
}

async function openChapter(chapterId: string) {
  if (!detail.value) return;
  const isChangingChapter = view.value === "reader";
  const response = await run("chapter", async () => {
    const book = detail.value!;
    const document = await networkNovelSource(book.source).loadDocument(book.id, chapterId);
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
    prefetchFollowingChapters(chapterId);
    if (isChangingChapter) {
      window.scrollTo({ top: 0 });
    } else {
      enterHistoryView("reader");
    }
  }
}

function prefetchFollowingChapters(chapterId: string) {
  if (!detail.value) return;
  const currentIndex = chapterIds.value.indexOf(chapterId);
  if (currentIndex < 0) return;

  const followingIds = chapterIds.value.slice(currentIndex + 1, currentIndex + 3);
  if (followingIds.length === 0) return;

  const book = detail.value;
  const source = networkNovelSource(book.source);
  void source.prefetchDocuments?.(book.id, followingIds).catch((error: unknown) => {
    console.warn("章节预取失败", error);
  });
}

function openNextChapter() {
  if (nextChapterId.value) void openChapter(nextChapterId.value);
}

async function toggleBookshelf() {
  if (!detail.value) return;
  const book = detail.value;
  await run("bookshelf", () => isOnBookshelf(book) ? removeBook(book) : addBook(book));
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

function back() {
  if (loading.value) {
    activeLoadSeq++;
    loading.value = false;
    loadingAction.value = null;
  }
  window.history.back();
}

onMounted(async () => {
  replaceHistoryView(view.value);
  window.addEventListener("popstate", handleHistoryBack);
  window.addEventListener("movel:android-back", handleAndroidBack);
  try {
    const [, sources] = await Promise.all([refreshBooks(), listNovelSources()]);
    sourceOptions.value = sources;
    selectedSource.value = sources[0]?.id ?? "";
  } catch (error) {
    errorMessage.value = describeError(error);
  } finally {
    bookshelfLoading.value = false;
  }
});

onBeforeUnmount(() => {
  stopThemeSync();
  delete document.documentElement.dataset.readerTheme;
  window.removeEventListener("popstate", handleHistoryBack);
  window.removeEventListener("movel:android-back", handleAndroidBack);
});
</script>

<template>
  <div class="page-bg">
    <header v-if="view === 'detail' || view === 'reader'" class="topbar">
      <div class="topbar-inner detail-topbar">
        <el-button class="back-button" :icon="ArrowLeft" round @click="back">
          {{ view === "reader" ? "返回目录" : `返回${lastLibraryView === "bookshelf" ? "书架" : "小说"}` }}
        </el-button>
      </div>
    </header>

    <main class="app-shell" :aria-busy="showLoadingOverlay">
      <el-alert
        v-if="errorMessage"
        class="error-alert"
        :title="errorMessage"
        type="error"
        show-icon
        closable
        @close="errorMessage = ''"
      />

      <NovelSearchView
        v-if="view === 'search'"
        v-model:query="query"
        v-model:selected-source="selectedSource"
        :source-options="sourceOptions"
        :results="results"
        :loading="loading"
        @search="search"
        @source-change="results = []"
        @open-novel="openNovel"
      />

      <BookshelfView
        v-else-if="view === 'bookshelf'"
        :books="books"
        :loading="loading"
        :bookshelf-loading="bookshelfLoading"
        @browse="openLibraryView('search')"
        @open-novel="openNovel"
      />

      <NovelDetailView
        v-else-if="view === 'detail' && detail"
        :detail="detail"
        :catalogue="catalogue"
        :loading="loading"
        :on-bookshelf="onBookshelf"
        :current-progress="currentProgress"
        @toggle-bookshelf="toggleBookshelf"
        @continue-reading="continueReading"
        @open-chapter="openChapter"
      />

      <NovelReader
        v-else-if="view === 'reader' && readerDocument"
        :document="readerDocument"
        :loading="loading"
        :initial-progress="readerInitialProgress"
        :has-next-chapter="Boolean(nextChapterId)"
        @next="openNextChapter"
        @progress="recordProgress"
      />
    </main>

    <LoadingOverlay :visible="showLoadingOverlay" :label="loadingLabel" />

    <nav v-if="view === 'search' || view === 'bookshelf'" class="view-dock" aria-label="主栏目">
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
