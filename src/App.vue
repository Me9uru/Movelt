<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import {
  ArrowLeft,
  Collection,
  Compass,
  Picture,
  User,
} from "@element-plus/icons-vue";
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
import LoadingOverlay from "./components/common/LoadingOverlay.vue";
import { useLibrary } from "./composables/useLibrary";
import { useDiscovery } from "./composables/useDiscovery";
import { login, logout, restoreUser, type LightNovelUser } from "./services/auth";
import type { AppRouteName, LibraryRouteName } from "./router";

type LoadingAction = "novel" | "chapter" | "bookshelf";

const route = useRoute();
const router = useRouter();
const view = computed<AppRouteName>(() => {
  const routeName = route.name;
  return routeName === "bookshelf" || routeName === "detail" || routeName === "reader"
    || routeName === "manga" || routeName === "manga-detail" || routeName === "manga-reader"
    ? routeName
    : "discovery";
});
const lastLibraryView = ref<LibraryRouteName>("discovery");
const detail = ref<NovelDetail | null>(null);
const catalogue = ref<Volume[]>([]);
const readerDocument = ref<ReaderDocument | null>(null);
const currentChapterId = ref<string | null>(null);
const resumeChapterId = ref<string | null>(null);
const loading = ref(false);
const loadingAction = ref<LoadingAction | null>(null);
const bookshelfLoading = ref(true);
const bookshelfQuery = ref("");
const bookshelfResults = ref<BookshelfEntry[] | null>(null);
const errorMessage = ref("");
const user = ref<LightNovelUser | null>(null);
const loginVisible = ref(false);
const loginEmail = ref("");
const loginPassword = ref("");
const loginLoading = ref(false);
const discovery = useDiscovery();
const {
  books,
  refreshBooks,
  searchBooks,
  addBook,
  removeBook,
  isOnBookshelf,
} = useLibrary();
const visibleBooks = computed(() => bookshelfResults.value ?? books.value);

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
  return currentIndex > 0 ? chapterIds.value[currentIndex - 1] ?? null : null;
});
const nextChapterId = computed(() => {
  if (!currentChapterId.value) return null;
  const currentIndex = chapterIds.value.indexOf(currentChapterId.value);
  return currentIndex >= 0 ? chapterIds.value[currentIndex + 1] ?? null : null;
});
const onBookshelf = computed(() => detail.value ? isOnBookshelf(detail.value) : false);
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

async function submitLogin() {
  if (!loginEmail.value || !loginPassword.value) return;
  loginLoading.value = true;
  try {
    user.value = await login(loginEmail.value, loginPassword.value);
    loginVisible.value = false;
    await Promise.all([refreshBooks(), discovery.initialize()]);
  } catch (error) {
    errorMessage.value = describeError(error);
  } finally {
    loginLoading.value = false;
  }
}

async function signOut() {
  await logout();
  user.value = null;
  books.value = [];
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
  if (view.value === "discovery" || view.value === "bookshelf") {
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
  void saveReadPosition(detail.value.id, readerDocument.value.serverChapterId, xpath).catch((error) => {
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
      params: { bookId: detail.value.id },
      query: route.query,
    });
  } else {
    void router.replace({ name: lastLibraryView.value });
  }
}

watch(
  () => [route.name, route.params.bookId, route.params.chapterId, route.query.from],
  async () => {
    const routeName = view.value;
    if (routeName === "manga" || routeName === "manga-detail" || routeName === "manga-reader") {
      errorMessage.value = "";
      return;
    }
    if (routeName === "discovery" || routeName === "bookshelf") {
      lastLibraryView.value = routeName;
      errorMessage.value = "";
      return;
    }

    if (route.query.from === "bookshelf" || route.query.from === "discovery") {
      lastLibraryView.value = route.query.from;
    }
    const source = lightNovelSourceId;
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
          params: { bookId },
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
  void restoreUser().then((value) => {
    user.value = value;
    if (value) {
      return refreshBooks().catch((error: unknown) => { errorMessage.value = describeError(error); });
    }
    return undefined;
  }).finally(() => { bookshelfLoading.value = false; });
  void discovery.initialize();
});

onBeforeUnmount(() => {
  window.removeEventListener("movel:android-back", handleAndroidBack);
});
</script>

<template>
  <div class="page-bg">
    <div class="auth-bar">
      <span v-if="user">{{ user.UserName }}</span>
      <el-button v-if="user" text @click="signOut">退出登录</el-button>
      <el-button v-else :icon="User" text @click="loginVisible = true">登录</el-button>
    </div>
    <header v-if="view === 'detail'" class="topbar">
      <div class="topbar-inner detail-topbar">
        <el-button class="back-button" :icon="ArrowLeft" @click="back">
          {{ `返回${lastLibraryView === "bookshelf" ? "书架" : "发现"}` }}
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
          @open-novel="openNovel"
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

        <component :is="Component" v-else />
      </RouterView>
    </main>

    <LoadingOverlay :visible="showLoadingOverlay" :label="loadingLabel" />

    <el-dialog v-model="loginVisible" title="登录 LightNovelShelf" width="min(420px, calc(100vw - 32px))" append-to-body>
      <el-form @submit.prevent="submitLogin">
        <el-form-item label="邮箱"><el-input v-model="loginEmail" autocomplete="email" /></el-form-item>
        <el-form-item label="密码"><el-input v-model="loginPassword" type="password" autocomplete="current-password" show-password /></el-form-item>
        <el-button type="primary" :loading="loginLoading" native-type="submit">登录</el-button>
      </el-form>
    </el-dialog>

    <nav v-if="view === 'discovery' || view === 'bookshelf' || view === 'manga'" class="view-dock" aria-label="主栏目">
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
      <button
        type="button"
        :class="{ active: view === 'manga' }"
        :aria-current="view === 'manga' ? 'page' : undefined"
        @click="openLibraryView('manga')"
      >
        <el-icon><Picture /></el-icon>
        <span>漫画</span>
      </button>
    </nav>
  </div>
</template>
