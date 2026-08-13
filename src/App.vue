<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import {
  ArrowLeft,
  Collection,
  Compass,
} from "@element-plus/icons-vue";
import { open } from "@tauri-apps/plugin-dialog";
import {
  getReaderOverview,
  type NovelDetail,
  type NovelSummary,
  type Volume,
} from "./services/novel";
import type { ReaderDocument } from "./domain/reader";
import type { BookshelfEntry } from "./domain/library";
import { networkNovelSource } from "./sources/networkNovel";
import { localEpubSource } from "./sources/localEpub";
import {
  canUseLocalEpubAssets,
  importEpub,
  localEpubSourceId,
} from "./services/localEpub";
import LoadingOverlay from "./components/common/LoadingOverlay.vue";
import { useLibrary } from "./composables/useLibrary";
import { useDiscovery } from "./composables/useDiscovery";
import type { AppRouteName, LibraryRouteName } from "./router";

type LoadingAction = "novel" | "chapter" | "bookshelf" | "import";

const route = useRoute();
const router = useRouter();
const view = computed<AppRouteName>(() => {
  const routeName = route.name;
  return routeName === "bookshelf" || routeName === "detail" || routeName === "reader"
    ? routeName
    : "discovery";
});
const lastLibraryView = ref<LibraryRouteName>("discovery");
const detail = ref<NovelDetail | null>(null);
const catalogue = ref<Volume[]>([]);
const readerDocument = ref<ReaderDocument | null>(null);
const currentChapterId = ref<string | null>(null);
const loading = ref(false);
const loadingAction = ref<LoadingAction | null>(null);
const bookshelfLoading = ref(true);
const bookshelfQuery = ref("");
const bookshelfResults = ref<BookshelfEntry[] | null>(null);
const errorMessage = ref("");
const discovery = useDiscovery();
const {
  books,
  refreshBooks,
  searchBooks,
  addBook,
  removeBook,
  isOnBookshelf,
  progressFor,
  loadProgress,
  saveProgress,
} = useLibrary();
const visibleBooks = computed(() => bookshelfResults.value ?? books.value);

function collectChapterIds(volumes: Volume[]): string[] {
  return volumes.flatMap((volume) => [
    ...volume.chapters.map((chapter) => chapter.id),
    ...collectChapterIds(volume.sections),
  ]);
}

function findChapter(volumes: Volume[], chapterId: string): { id: string; title: string } | undefined {
  for (const volume of volumes) {
    const chapter = volume.chapters.find((item) => item.id === chapterId);
    if (chapter) return chapter;
    const nested = findChapter(volume.sections, chapterId);
    if (nested) return nested;
  }
  return undefined;
}

const chapterIds = computed(() => collectChapterIds(catalogue.value));
const previousChapterId = computed(() => {
  if (!currentChapterId.value) return null;
  const currentIndex = chapterIds.value.indexOf(currentChapterId.value);
  return currentIndex > 0 ? chapterIds.value[currentIndex - 1] ?? null : null;
});
const nextChapterId = computed(() => {
  if (!currentChapterId.value) return null;
  const currentIndex = chapterIds.value.indexOf(currentChapterId.value);
  return currentIndex >= 0 ? chapterIds.value[currentIndex + 1] ?? null : null;
});
const onBookshelf = computed(() => detail.value ? isOnBookshelf(detail.value) : false);
const currentProgress = computed(() => {
  const saved = detail.value ? progressFor(detail.value) : null;
  if (!saved) return null;
  return {
    ...saved,
    bookLocation: bookLocation(saved.documentId, saved.location, saved.bookLocation),
  };
});
function bookLocation(chapterId: string, location: number, fallback = 0): number {
  const chapterIndex = chapterIds.value.indexOf(chapterId);
  if (chapterIndex < 0 || chapterIds.value.length === 0) return fallback;
  const chapterLocation = Math.min(1, Math.max(0, location));
  return (chapterIndex + chapterLocation) / chapterIds.value.length;
}
const readerInitialProgress = computed(() => {
  const progress = currentProgress.value;
  return progress?.documentId === currentChapterId.value ? progress : null;
});
const loadingCopy = computed(() => {
  switch (loadingAction.value) {
    case "novel":
      return { title: "正在加载作品详情", hint: "正在获取简介与章节目录" };
    case "chapter":
      return { title: "正在加载章节", hint: "内容较多时可能需要稍候" };
    case "bookshelf":
      return { title: "正在更新书架", hint: "请稍候" };
    case "import":
      return { title: "正在导入 EPUB", hint: "正在保存并解析书籍" };
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

async function searchShelf() {
  const searchQuery = bookshelfQuery.value.trim();
  if (!searchQuery) {
    bookshelfResults.value = null;
    return;
  }
  const response = await run("bookshelf", () => searchBooks(searchQuery));
  if (response) bookshelfResults.value = response;
}

function openLibraryView(nextView: LibraryRouteName) {
  lastLibraryView.value = nextView;
  errorMessage.value = "";
  void router.replace({ name: nextView });
}

function handleAndroidBack(event: Event) {
  if (view.value === "detail" || view.value === "reader") {
    event.preventDefault();
    back();
  }
}

async function loadNovel(source: string, novelId: string): Promise<boolean> {
  const response = await run("novel", async () => {
    const overview = await getReaderOverview(source, novelId);
    const progress = await loadProgress(overview.detail);
    if (overview.detail.source === localEpubSourceId && progress) {
      const chapter = findChapter(overview.volumes, progress.documentId);
      if (chapter && chapter.title !== progress.documentTitle) {
        await saveProgress(overview.detail, {
          documentId: progress.documentId,
          documentTitle: chapter.title,
          location: progress.location,
          bookLocation: progress.bookLocation,
        });
      }
    }
    return overview;
  });
  if (response) {
    detail.value = response.detail;
    catalogue.value = response.volumes;
    return true;
  }
  return false;
}

async function openNovel(novel: NovelSummary) {
  if (view.value === "discovery" || view.value === "bookshelf") {
    lastLibraryView.value = view.value;
  }
  if (await loadNovel(novel.source, novel.id)) {
    await router.push({
      name: "detail",
      params: { source: novel.source, bookId: novel.id },
      query: { from: lastLibraryView.value },
    });
  }
}

async function openChapter(chapterId: string, navigate = true) {
  if (!detail.value) return;
  const isChangingChapter = view.value === "reader";
  const response = await run("chapter", async () => {
    const book = detail.value!;
    const source = book.source === localEpubSourceId ? localEpubSource : networkNovelSource(book.source);
    const document = await source.loadDocument(book.id, chapterId, findChapter(catalogue.value, chapterId)?.title);
    const existing = progressFor(book);
    await saveProgress(book, {
      documentId: chapterId,
      documentTitle: document.title,
      location: existing?.documentId === chapterId ? existing.location : 0,
      bookLocation: bookLocation(
        chapterId,
        existing?.documentId === chapterId ? existing.location : 0,
      ),
    });
    return document;
  });
  if (response) {
    readerDocument.value = response;
    currentChapterId.value = chapterId;
    if (!navigate) return;
    if (isChangingChapter) {
      await router.replace({
        name: "reader",
        params: {
          source: detail.value.source,
          bookId: detail.value.id,
          chapterId,
        },
        query: route.query,
      });
    } else {
      await router.push({
        name: "reader",
        params: {
          source: detail.value.source,
          bookId: detail.value.id,
          chapterId,
        },
        query: route.query,
      });
    }
  }
}

function openNextChapter() {
  if (nextChapterId.value) void openChapter(nextChapterId.value);
}

function openPreviousChapter() {
  if (previousChapterId.value) void openChapter(previousChapterId.value);
}

async function toggleBookshelf() {
  if (!detail.value) return;
  const book = detail.value;
  await run("bookshelf", async () => {
    await (isOnBookshelf(book) ? removeBook(book) : addBook(book));
    const searchQuery = bookshelfQuery.value.trim();
    if (bookshelfResults.value && searchQuery) {
      bookshelfResults.value = await searchBooks(searchQuery);
    }
  });
}

async function chooseAndImportEpub() {
  if (!canUseLocalEpubAssets()) {
    errorMessage.value = "导入 EPUB 仅支持桌面应用；浏览器调试可查看已导入的书籍。";
    return;
  }
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "EPUB", extensions: ["epub"] }],
  });
  if (!selected || Array.isArray(selected)) return;
  const overview = await run("import", () => importEpub(selected));
  if (!overview) return;
  await refreshBooks();
  detail.value = overview.detail;
  catalogue.value = overview.volumes;
  await loadProgress(overview.detail);
  lastLibraryView.value = "bookshelf";
  await router.push({
    name: "detail",
    params: { source: overview.detail.source, bookId: overview.detail.id },
    query: { from: "bookshelf" },
  });
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
    bookLocation: bookLocation(currentChapterId.value, location),
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
  if (window.history.state?.back) {
    router.back();
  } else if (view.value === "reader" && detail.value) {
    void router.replace({
      name: "detail",
      params: { source: detail.value.source, bookId: detail.value.id },
      query: route.query,
    });
  } else {
    void router.replace({ name: lastLibraryView.value });
  }
}

watch(
  () => [route.name, route.params.source, route.params.bookId, route.params.chapterId, route.query.from],
  async () => {
    const routeName = view.value;
    if (routeName === "discovery" || routeName === "bookshelf") {
      lastLibraryView.value = routeName;
      errorMessage.value = "";
      return;
    }

    if (route.query.from === "bookshelf" || route.query.from === "discovery") {
      lastLibraryView.value = route.query.from;
    }
    const source = typeof route.params.source === "string" ? route.params.source : "";
    const bookId = typeof route.params.bookId === "string" ? route.params.bookId : "";
    if (!source || !bookId) {
      await router.replace({ name: lastLibraryView.value });
      return;
    }

    if (detail.value?.source !== source || detail.value.id !== bookId) {
      const loaded = await loadNovel(source, bookId);
      if (!loaded) return;
    }
    if (routeName === "reader") {
      const chapterId = typeof route.params.chapterId === "string" ? route.params.chapterId : "";
      if (!chapterId) {
        await router.replace({
          name: "detail",
          params: { source, bookId },
          query: route.query,
        });
      } else if (!readerDocument.value || currentChapterId.value !== chapterId) {
        await openChapter(chapterId, false);
      }
    }
  },
  { immediate: true },
);

onMounted(() => {
  window.addEventListener("movel:android-back", handleAndroidBack);
  void discovery.initialize();
  void refreshBooks()
    .catch((error: unknown) => {
      errorMessage.value = describeError(error);
    })
    .finally(() => {
      bookshelfLoading.value = false;
    });
});

onBeforeUnmount(() => {
  window.removeEventListener("movel:android-back", handleAndroidBack);
});
</script>

<template>
  <div class="page-bg">
    <header v-if="view === 'detail' || view === 'reader'" class="topbar">
      <div class="topbar-inner detail-topbar">
        <el-button class="back-button" :icon="ArrowLeft" @click="back">
          {{ view === "reader" ? "返回目录" : `返回${lastLibraryView === "bookshelf" ? "书架" : "发现"}` }}
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

      <RouterView v-slot="{ Component }">
        <component
          :is="Component"
          v-if="view === 'discovery'"
          v-model:ranking-sort="discovery.rankingSort.value"
          v-model:custom-tag="discovery.customTag.value"
          v-model:search-query="discovery.searchQuery.value"
          :unavailable-message="discovery.unavailableMessage.value"
          :recommendations="discovery.recommendations.value"
          :ranking="discovery.ranking.value"
          :category="discovery.category.value"
          :category-tag="discovery.categoryTag.value"
          :search-result="discovery.search.value"
          :loading="discovery.loading.value"
          :errors="discovery.errors.value"
          @initialize="discovery.initialize"
          @retry-recommendations="discovery.loadRecommendations"
          @load-ranking="discovery.loadRanking"
          @load-category="discovery.loadCategory"
          @search="discovery.runSearch"
          @select-category="discovery.selectCategory"
          @open-novel="openNovel"
        />

        <component
          :is="Component"
          v-else-if="view === 'bookshelf'"
          v-model:query="bookshelfQuery"
          :books="visibleBooks"
          :total-books="books.length"
          :search-active="bookshelfResults !== null"
          :loading="loading"
          :bookshelf-loading="bookshelfLoading"
          @search="searchShelf"
          @browse="openLibraryView('discovery')"
          @import-epub="chooseAndImportEpub"
          @open-novel="openNovel"
        />

        <component
          :is="Component"
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

        <component
          :is="Component"
          v-else-if="view === 'reader' && readerDocument"
          :document="readerDocument"
          :loading="loading"
          :initial-progress="readerInitialProgress"
          :has-previous-chapter="Boolean(previousChapterId)"
          :has-next-chapter="Boolean(nextChapterId)"
          @previous="openPreviousChapter"
          @next="openNextChapter"
          @progress="recordProgress"
        />
      </RouterView>
    </main>

    <LoadingOverlay :visible="showLoadingOverlay" :label="loadingLabel" />

    <nav v-if="view === 'discovery' || view === 'bookshelf'" class="view-dock" aria-label="主栏目">
      <button
        type="button"
        :class="{ active: view === 'discovery' }"
        :aria-current="view === 'discovery' ? 'page' : undefined"
        @click="openLibraryView('discovery')"
      >
        <el-icon><Compass /></el-icon>
        <span>发现</span>
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
