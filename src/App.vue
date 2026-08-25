<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import {
  getReaderDocument,
  getReaderOverview,
  lightNovelSourceId,
  saveReadPosition,
} from "./services/novel";
import type {
  NovelSummary,
  ReaderDocument,
  Volume,
} from "./domain/novel";
import type { ReadPosition } from "./domain/readPosition";
import type { MangaSummary } from "./domain/manga";
import type { BookshelfEntry } from "./domain/bookshelf";
import { listMangaBookshelf } from "./services/manga";
import LoadingOverlay from "./components/common/LoadingOverlay.vue";
import MainNavigation from "./components/layout/MainNavigation.vue";
import { useLibrary } from "./composables/useLibrary";
import { novelDiscoveryAdapter } from "./composables/discoveryAdapters";
import { useDiscovery } from "./composables/useDiscovery";
import { useReaderSettings } from "./composables/useReaderSettings";
import { useAuthStore } from "./stores/auth";
import type { AppRouteName, LibraryRouteName, ReturnRouteName } from "./types/router";
import type { LoadingAction } from "./types/loading";
import { showError } from "./utils/error";

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
    routeName === "settings" ||
    routeName === "login"
    ? routeName
    : "novels";
});
const backLabel = computed(() =>
  view.value === "manga-detail"
    ? "返回漫画"
    : `返回${lastLibraryView.value === "bookshelf" ? "书架" : "小说"}`,
);

const lastLibraryView = ref<ReturnRouteName>("novels");
const detail = ref<NovelSummary | null>(null);
const catalogue = ref<Volume[]>([]);
const readerDocument = ref<ReaderDocument | null>(null);
const currentChapterId = ref<string | null>(null);
const resumeChapterId = ref<string | null>(null);
const resumeReadPosition = ref<ReadPosition | null>(null);
const readerChapterEntry = ref<"default" | "next" | "previous">("default");
const readerRenderKey = ref(0);
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
const auth = useAuthStore();
const discovery = useDiscovery(novelDiscoveryAdapter);
const { settings: readerSettings } = useReaderSettings("novel");
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
const showLoadingOverlay = computed(() => loading.value);
const contentLoading = computed(
  () => loading.value || (view.value === "bookshelf" && bookshelfLoading.value),
);
const loadingLabel = computed(() => loadingCopy.value.title);

let activeLoadSeq = 0;
let readerRestoreInFlight = false;

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

  const task =
    bookshelfKind.value === "novel"
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

function backFromManga() {
  if (window.history.state?.back) {
    router.back();
  } else {
    void router.replace({ name: "manga" });
  }
}

function handleAndroidBack(event: Event) {
  if (view.value === "detail" || view.value === "reader") {
    event.preventDefault();
    back();
  } else if (view.value === "manga-detail") {
    event.preventDefault();
    backFromManga();
  }
}

function handleAuthenticationExpired() {
  auth.expire();
  redirectToLogin();
}

async function restoreReaderAfterForeground() {
  if (
    document.visibilityState !== "visible" ||
    readerRestoreInFlight ||
    view.value !== "reader" ||
    !detail.value ||
    !readerDocument.value ||
    !currentChapterId.value
  ) {
    return;
  }

  readerRestoreInFlight = true;
  try {
    // Android may discard WebView-rendered nodes while keeping the Vue state.
    // Re-fetch the active document and remount the reader so v-html is inserted again.
    await openChapter(currentChapterId.value, false);
    readerRenderKey.value++;
  } finally {
    readerRestoreInFlight = false;
  }
}

function handleDocumentVisibilityChange() {
  if (document.visibilityState === "visible") {
    void restoreReaderAfterForeground();
  }
}

function redirectToLogin() {
  if (route.name === "login") return;
  void router.replace({
    name: "login",
    query: { redirect: route.fullPath },
  });
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
    resumeReadPosition.value = response.readPosition;
    return true;
  }
  return false;
}

async function openNovel(novel: NovelSummary) {
  if (view.value === "novels") {
    lastLibraryView.value = route.name === "novel-search" ? "novel-search" : "novels";
  } else if (view.value === "bookshelf") {
    lastLibraryView.value = "bookshelf";
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

async function openChapter(
  chapterId: string,
  navigate = true,
  entry: "default" | "next" | "previous" = "default",
) {
  if (!detail.value) return;
  readerChapterEntry.value = entry;
  const isChangingChapter = view.value === "reader";
  const response = await run("chapter", async () => {
    return getReaderDocument(
      detail.value!.source,
      detail.value!.id,
      chapterId,
      readerSettings.convert,
    );
  });
  if (response) {
    readerDocument.value = response;
    currentChapterId.value = chapterId;
    if (!navigate) return;
    try {
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
    } catch {
      // router.onError has already presented the route-loading failure in a dialog.
    }
  }
}

function openNextChapter() {
  if (nextChapterId.value) void openChapter(nextChapterId.value, true, "next");
}

function openPreviousChapter() {
  if (previousChapterId.value) void openChapter(previousChapterId.value, true, "previous");
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
  resumeChapterId.value = currentChapterId.value;
  resumeReadPosition.value = {
    chapterId: currentChapterId.value,
    position: xpath,
  };
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
    auth.user,
  ],
  async () => {
    if (!auth.user) return;
    const routeName = view.value;
    if (
      routeName === "manga" ||
      routeName === "manga-detail" ||
      routeName === "manga-reader" ||
      routeName === "settings" ||
      routeName === "login"
    ) {
      return;
    }
    if (routeName === "novels" || routeName === "bookshelf") {
      lastLibraryView.value =
        route.name === "novel-search" ? "novel-search" : routeName;
      if (routeName === "bookshelf") loadActiveBookshelf();
      return;
    }

    if (
      route.query.from === "bookshelf" ||
      route.query.from === "novels" ||
      route.query.from === "novel-search"
    ) {
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
      redirectToLogin();
    }
  },
);

watch(
  () => readerSettings.convert,
  () => {
    if (view.value === "reader" && currentChapterId.value) {
      void openChapter(currentChapterId.value, false);
    }
  },
);

onMounted(() => {
  window.addEventListener("movel:android-back", handleAndroidBack);
  document.addEventListener("visibilitychange", handleDocumentVisibilityChange);
  window.addEventListener(
    "movel:authentication-expired",
    handleAuthenticationExpired,
  );
  void auth
    .restore()
    .then((value) => {
      if (value) {
        if (view.value === "bookshelf") loadActiveBookshelf();
        return undefined;
      }
      redirectToLogin();
      return undefined;
    })
    .catch((error: unknown) => {
      // A missing or unavailable system credential store must not leave the
      // application without a visible way to authenticate.
      redirectToLogin();
      showError(error);
    });
});

onBeforeUnmount(() => {
  window.removeEventListener("movel:android-back", handleAndroidBack);
  document.removeEventListener("visibilitychange", handleDocumentVisibilityChange);
  window.removeEventListener(
    "movel:authentication-expired",
    handleAuthenticationExpired,
  );
});
</script>

<template>
  <div class="page-bg">
    <RouterView v-slot="{ Component }">
      <component :is="Component" v-if="view === 'login'" />

      <main v-else-if="auth.user" class="app-shell" :aria-busy="contentLoading">
        <component
          :is="Component"
          v-if="view === 'novels'"
          v-model:ranking-days="discovery.rankingDays.value"
          v-model:search-query="discovery.searchQuery.value"
          v-model:search-mode="discovery.searchMode.value"
          :unavailable-message="discovery.unavailableMessage.value"
          :recommendations="discovery.recommendations.value"
          :ranking="discovery.ranking.value"
          :search-result="discovery.search.value"
          :loading="discovery.loading.value"
          :errors="discovery.errors.value"
          @initialize="discovery.initialize"
          @retry-recommendations="discovery.loadRecommendations"
          @load-ranking="discovery.loadRanking"
          @search="discovery.runSearch"
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
          :back-label="backLabel"
          @back="back"
          @toggle-bookshelf="toggleBookshelf"
          @continue-reading="continueReading"
          @open-chapter="openChapter"
        />

        <component
          :is="Component"
          v-else-if="view === 'reader' && readerDocument"
          :key="`${readerDocument.id}:${readerRenderKey}`"
          :document="readerDocument"
          :resume-position="resumeReadPosition"
          :chapter-entry="readerChapterEntry"
          :loading="loading"
          :has-previous-chapter="Boolean(previousChapterId)"
          :has-next-chapter="Boolean(nextChapterId)"
          @previous="openPreviousChapter"
          @next="openNextChapter"
          @progress="recordProgress"
        />

        <component :is="Component" v-else-if="view === 'settings'" />

        <component
          :is="Component"
          v-else-if="view !== 'detail' && view !== 'reader'"
        />
      </main>
    </RouterView>

    <LoadingOverlay :visible="showLoadingOverlay" :label="loadingLabel" />

    <MainNavigation
      v-if="auth.user"
      :view="view"
      @navigate="openLibraryView"
    />
  </div>
</template>
