<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import { ArrowLeft } from "@element-plus/icons-vue";
import {
  getReaderDocument,
  getReaderOverview,
  lightNovelSourceId,
  saveReadPosition,
  type NovelDetail,
  type NovelSummary,
  type Volume,
} from "./services/novel";
import type { ReaderDocument } from "./domain/reader";
import type { BookshelfEntry } from "./services/library";
import { listMangaBookshelf, type MangaSummary } from "./services/manga";
import LoadingOverlay from "./components/common/LoadingOverlay.vue";
import AuthDialog from "./components/auth/AuthDialog.vue";
import MainNavigation from "./components/layout/MainNavigation.vue";
import { useLibrary } from "./composables/useLibrary";
import { useDiscovery } from "./composables/useDiscovery";
import { useAuthStore } from "./stores/auth";
import type { AppRouteName, LibraryRouteName } from "./router";
import { showError } from "./utils/error";

type LoadingAction = "novel" | "chapter" | "bookshelf";

const route = useRoute();
const router = useRouter();
const view = computed<AppRouteName>(() => {
  const routeName = route.name;
  return routeName === "bookshelf" ||
    routeName === "detail" ||
    routeName === "reader" ||
    routeName === "manga" ||
    routeName === "manga-detail" ||
    routeName === "manga-reader" ||
    routeName === "settings"
    ? routeName
    : "novels";
});
const lastLibraryView = ref<LibraryRouteName>("novels");
const detail = ref<NovelDetail | null>(null);
const catalogue = ref<Volume[]>([]);
const readerDocument = ref<ReaderDocument | null>(null);
const currentChapterId = ref<string | null>(null);
const resumeChapterId = ref<string | null>(null);
const loading = ref(false);
const loadingAction = ref<LoadingAction | null>(null);
const bookshelfQuery = ref("");
const bookshelfResults = ref<BookshelfEntry[] | null>(null);
const mangaBooks = ref<MangaSummary[]>([]);
const bookshelfKind = ref<"novel" | "manga">("novel");
const novelBookshelfLoading = ref(false);
const novelBookshelfLoaded = ref(false);
const mangaBookshelfLoading = ref(false);
const mangaBookshelfLoaded = ref(false);
const loginVisible = ref(false);
const auth = useAuthStore();
const discovery = useDiscovery();
const { books, refreshBooks, searchBooks, addBook, removeBook, isOnBookshelf } =
  useLibrary();
const visibleBooks = computed(() => bookshelfResults.value ?? books.value);
const bookshelfLoading = computed(
  () => novelBookshelfLoading.value || mangaBookshelfLoading.value,
);

async function refreshNovelBookshelf() {
  if (novelBookshelfLoading.value || novelBookshelfLoaded.value) return;

  novelBookshelfLoading.value = true;
  try {
    await refreshBooks();
    novelBookshelfLoaded.value = true;
  } finally {
    novelBookshelfLoading.value = false;
  }
}

async function refreshMangaBookshelf() {
  if (mangaBookshelfLoading.value || mangaBookshelfLoaded.value) return;

  mangaBookshelfLoading.value = true;
  try {
    mangaBooks.value = await listMangaBookshelf();
    mangaBookshelfLoaded.value = true;
  } finally {
    mangaBookshelfLoading.value = false;
  }
}

function collectChapterIds(volumes: Volume[]): string[] {
  return volumes.flatMap((volume) => [
    ...volume.chapters.map((chapter) => chapter.id),
    ...collectChapterIds(volume.sections),
  ]);
}

const chapterIds = computed(() => collectChapterIds(catalogue.value));
const previousChapterId = computed(() => {
  if (!currentChapterId.value) return null;
  const currentIndex = chapterIds.value.indexOf(currentChapterId.value);
  return currentIndex > 0 ? (chapterIds.value[currentIndex - 1] ?? null) : null;
});
const nextChapterId = computed(() => {
  if (!currentChapterId.value) return null;
  const currentIndex = chapterIds.value.indexOf(currentChapterId.value);
  return currentIndex >= 0
    ? (chapterIds.value[currentIndex + 1] ?? null)
    : null;
});
const onBookshelf = computed(() =>
  detail.value ? isOnBookshelf(detail.value) : false,
);
const loadingCopy = computed(() => {
  switch (loadingAction.value) {
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
const showLoadingOverlay = computed(
  () => loading.value,
);
const contentLoading = computed(
  () => loading.value || (view.value === "bookshelf" && bookshelfLoading.value),
);
const loadingLabel = computed(() => loadingCopy.value.title);

let activeLoadSeq = 0;

async function run<T>(
  action: LoadingAction,
  task: () => Promise<T>,
): Promise<T | null> {
  if (loading.value) return null;
  const seq = ++activeLoadSeq;
  loading.value = true;
  loadingAction.value = action;
  try {
    const result = await task();
    if (seq !== activeLoadSeq) return null;
    return result;
  } catch (error) {
    if (seq !== activeLoadSeq) return null;
    showError(error);
    return null;
  } finally {
    if (seq === activeLoadSeq) {
      loading.value = false;
      loadingAction.value = null;
    }
  }
}

async function searchShelf() {
  if (bookshelfKind.value === "manga") return;
  const searchQuery = bookshelfQuery.value.trim();
  if (!searchQuery) {
    bookshelfResults.value = null;
    return;
  }
  const response = await run("bookshelf", () => searchBooks(searchQuery));
  if (response) bookshelfResults.value = response;
}

function changeBookshelfKind(kind: "novel" | "manga") {
  bookshelfKind.value = kind;
  bookshelfQuery.value = "";
  bookshelfResults.value = null;
  loadActiveBookshelf();
}

function loadActiveBookshelf() {
  if (!auth.user) return;

  const task = bookshelfKind.value === "novel"
    ? refreshNovelBookshelf()
    : refreshMangaBookshelf();
  void task.catch((error: unknown) => {
    showError(error);
  });
}

function openLibraryView(nextView: LibraryRouteName) {
  lastLibraryView.value = nextView;
  if (nextView === "bookshelf") loadActiveBookshelf();
  void router.replace({ name: nextView });
}

async function handleAuthenticated() {
  if (view.value === "bookshelf") loadActiveBookshelf();
}

function handleAndroidBack(event: Event) {
  if (view.value === "detail" || view.value === "reader") {
    event.preventDefault();
    back();
  }
}

function handleAuthenticationExpired() {
  auth.expire();
  loginVisible.value = true;
}

async function loadNovel(source: string, novelId: string): Promise<boolean> {
  const response = await run("novel", async () => {
    const overview = await getReaderOverview(source, novelId);
    return overview;
  });
  if (response) {
    detail.value = response.detail;
    catalogue.value = response.volumes;
    resumeChapterId.value = response.readPosition?.chapterId ?? null;
    return true;
  }
  return false;
}

async function openNovel(novel: NovelSummary) {
  if (view.value === "novels" || view.value === "bookshelf") {
    lastLibraryView.value = view.value;
  }
  if (await loadNovel(novel.source, novel.id)) {
    await router.push({
      name: "detail",
      params: { bookId: novel.id },
      query: { from: lastLibraryView.value },
    });
  }
}

function openManga(manga: MangaSummary) {
  void router.push({ name: "manga-detail", params: { mangaId: manga.id } });
}

async function openChapter(chapterId: string, navigate = true) {
  if (!detail.value) return;
  const isChangingChapter = view.value === "reader";
  const response = await run("chapter", async () => {
    return getReaderDocument(detail.value!.source, detail.value!.id, chapterId);
  });
  if (response) {
    readerDocument.value = response;
    currentChapterId.value = chapterId;
    if (!navigate) return;
    if (isChangingChapter) {
      await router.replace({
        name: "reader",
        params: {
          bookId: detail.value.id,
          chapterId,
        },
        query: route.query,
      });
    } else {
      await router.push({
        name: "reader",
        params: {
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

function continueReading() {
  if (resumeChapterId.value) void openChapter(resumeChapterId.value);
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

function recordProgress(xpath: string) {
  if (!detail.value || !readerDocument.value || !currentChapterId.value) return;
  void saveReadPosition(
    detail.value.id,
    readerDocument.value.serverChapterId,
    xpath,
  ).catch((error) => {
    showError(error);
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
      params: { bookId: detail.value.id },
      query: route.query,
    });
  } else {
    void router.replace({ name: lastLibraryView.value });
  }
}

watch(
  () => [
    route.name,
    route.params.bookId,
    route.params.chapterId,
    route.query.from,
  ],
  async () => {
    const routeName = view.value;
    if (
      routeName === "manga" ||
      routeName === "manga-detail" ||
      routeName === "manga-reader" ||
      routeName === "settings"
    ) {
      return;
    }
    if (routeName === "novels" || routeName === "bookshelf") {
      lastLibraryView.value = routeName;
      if (routeName === "bookshelf") loadActiveBookshelf();
      return;
    }

    if (route.query.from === "bookshelf" || route.query.from === "novels") {
      lastLibraryView.value = route.query.from;
    }
    const source = lightNovelSourceId;
    const bookId =
      typeof route.params.bookId === "string" ? route.params.bookId : "";
    if (!source || !bookId) {
      await router.replace({ name: lastLibraryView.value });
      return;
    }

    if (detail.value?.source !== source || detail.value.id !== bookId) {
      const loaded = await loadNovel(source, bookId);
      if (!loaded) return;
    }
    if (routeName === "reader") {
      const chapterId =
        typeof route.params.chapterId === "string"
          ? route.params.chapterId
          : "";
      if (!chapterId) {
        await router.replace({
          name: "detail",
          params: { bookId },
          query: route.query,
        });
      } else if (
        !readerDocument.value ||
        currentChapterId.value !== chapterId
      ) {
        await openChapter(chapterId, false);
      }
    }
  },
  { immediate: true },
);

watch(
  () => auth.user,
  (user, previousUser) => {
    if (!user && previousUser) {
      books.value = [];
      mangaBooks.value = [];
      novelBookshelfLoaded.value = false;
      mangaBookshelfLoaded.value = false;
      bookshelfResults.value = null;
      loginVisible.value = true;
    }
  },
);

onMounted(() => {
  window.addEventListener("movel:android-back", handleAndroidBack);
  window.addEventListener("movel:authentication-expired", handleAuthenticationExpired);
  void auth
    .restore()
    .then((value) => {
      if (value) {
        if (view.value === "bookshelf") loadActiveBookshelf();
        return undefined;
      }
      loginVisible.value = true;
      return undefined;
    });
});

onBeforeUnmount(() => {
  window.removeEventListener("movel:android-back", handleAndroidBack);
  window.removeEventListener("movel:authentication-expired", handleAuthenticationExpired);
});
</script>

<template>
  <div class="page-bg">
    <header v-if="view === 'detail'" class="topbar">
      <div class="topbar-inner detail-topbar">
        <el-button class="back-button" :icon="ArrowLeft" @click="back">
          {{ `返回${lastLibraryView === "bookshelf" ? "书架" : "小说"}` }}
        </el-button>
      </div>
    </header>

    <main class="app-shell" :aria-busy="contentLoading">
      <RouterView v-if="auth.user" v-slot="{ Component }">
        <component
          :is="Component"
          v-if="view === 'novels'"
          v-model:ranking-days="discovery.rankingDays.value"
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
          :manga="mangaBooks"
          :active-kind="bookshelfKind"
          :search-active="bookshelfResults !== null"
          :loading="loading"
          :bookshelf-loading="bookshelfLoading || mangaBookshelfLoading"
          @search="searchShelf"
          @browse="
            openLibraryView(bookshelfKind === 'novel' ? 'novels' : 'manga')
          "
          @open-novel="openNovel"
          @open-manga="openManga"
          @update:active-kind="changeBookshelfKind"
        />

        <component
          :is="Component"
          v-else-if="view === 'detail' && detail"
          :detail="detail"
          :catalogue="catalogue"
          :loading="loading"
          :on-bookshelf="onBookshelf"
          :resume-chapter-id="resumeChapterId"
          @toggle-bookshelf="toggleBookshelf"
          @continue-reading="continueReading"
          @open-chapter="openChapter"
        />

        <component
          :is="Component"
          v-else-if="view === 'reader' && readerDocument"
          :document="readerDocument"
          :loading="loading"
          :has-previous-chapter="Boolean(previousChapterId)"
          :has-next-chapter="Boolean(nextChapterId)"
          @previous="openPreviousChapter"
          @next="openNextChapter"
          @progress="recordProgress"
        />

        <component
          :is="Component"
          v-else-if="view === 'settings'"
          @login="loginVisible = true"
        />

        <component
          :is="Component"
          v-else-if="view !== 'detail' && view !== 'reader'"
        />
      </RouterView>
    </main>

    <LoadingOverlay :visible="showLoadingOverlay" :label="loadingLabel" />

    <AuthDialog
      v-model:visible="loginVisible"
      @authenticated="handleAuthenticated"
    />
    <MainNavigation
      :view="view"
      :book-count="books.length"
      @navigate="openLibraryView"
    />
  </div>
</template>
