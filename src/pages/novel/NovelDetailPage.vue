<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import type { NovelChapterSummary, NovelDetail } from "../../domain/novel";
import BookChapterList from "../../components/book/BookChapterList.vue";
import type { BookChapterItem } from "../../types/book";
import BookDetailLayout from "../../layout/BookDetailLayout.vue";
import LoadingOverlay from "../../components/common/LoadingOverlay.vue";
import ErrorState from "../../components/common/ErrorState.vue";
import { useBookshelfStore } from "../../stores/bookshelf";
import { getReaderOverview, lightNovelSourceId } from "../../services/novel";
import { getErrorMessage, showError } from "../../utils/error";

const route = useRoute();
const router = useRouter();
const bookshelf = useBookshelfStore();
const detail = ref<NovelDetail | null>(null);
const chapters = ref<NovelChapterSummary[]>([]);
const resumeChapterId = ref<string | null>(null);
const loading = ref(true);
const error = ref("");
const bookId = computed(() =>
  typeof route.params.bookId === "string" ? route.params.bookId : "",
);
const from = computed(() =>
  route.query.from === "bookshelf" ? route.query.from : "novels",
);

const backLabel = computed(
  () => `返回${from.value === "bookshelf" ? "书架" : "小说"}`,
);

const onBookshelf = computed(() =>
  detail.value ? bookshelf.isOnBookshelf(detail.value) : false,
);

const load = async (): Promise<void> => {
  if (!bookId.value) return;
  loading.value = true;
  error.value = "";
  try {
    const overview = await getReaderOverview(lightNovelSourceId, bookId.value);
    detail.value = overview;
    chapters.value = overview.chapters;
    resumeChapterId.value = overview.readPosition?.chapterId ?? null;
  } catch (value) {
    error.value = getErrorMessage(value, "无法加载作品详情");
    showError(value, "无法加载作品详情");
  } finally {
    loading.value = false;
  }
};

const goBack = (): void => {
  if (window.history.state?.back) router.back();
  else void router.replace({
    name: from.value,
    query: from.value === "novels"
      ? {
          tab: route.query.tab,
          q: route.query.q,
          mode: route.query.mode,
        }
      : {},
  });
};

const openChapter = (chapterId: string): void => {
  if (!detail.value) return;
  void router.push({
    name: "novel-reader",
    params: { bookId: detail.value.id, chapterId },
    query: route.query,
  });
};

const continueReading = (): void => {
  const chapterId = resumeChapterId.value ?? chapters.value[0]?.id;
  if (chapterId) openChapter(chapterId);
};

const toggleBookshelf = async (): Promise<void> => {
  if (!detail.value) return;
  loading.value = true;
  try {
    if (onBookshelf.value) await bookshelf.removeBook(detail.value);
    else await bookshelf.addBook(detail.value);
  } catch (value) {
    showError(value, "更新书架失败");
  } finally {
    loading.value = false;
  }
};

onMounted(() => void load());
watch(bookId, () => void load());

const chapterCount = computed(() => chapters.value.length);
const stats = computed(() => [
  { value: chapterCount.value, label: "话" },
  ...(detail.value?.updated_at
    ? [{ label: `更新于 ${detail.value.updated_at}` }]
    : []),
]);

const toChapterItem = (
  chapter: NovelChapterSummary,
): BookChapterItem<NovelChapterSummary> => {
  return { id: chapter.id, title: chapter.title, data: chapter };
};

const chapterItems = computed(() => chapters.value.map(toChapterItem));
</script>

<template>
  <div class="book-detail-page">
    <header class="topbar book-detail-topbar">
      <div class="topbar-inner detail-topbar">
        <var-button text size="large" :elevation="false" @click="goBack"
          ><var-icon name="arrow-left" />{{ backLabel }}</var-button
        >
      </div>
    </header>
    <section class="detail-view">
      <LoadingOverlay
        v-if="loading && !detail"
        inline
        visible
        label="正在加载作品详情"
      />
      <ErrorState
        v-else-if="error"
        title="作品详情加载失败"
        :message="error"
        :loading="loading"
        @retry="load"
      />
      <BookDetailLayout
        v-else-if="detail"
        :title="detail.title"
        :cover-url="detail.cover_url"
        :author="detail.author || '佚名'"
        :status="detail.status"
        :tags="detail.tags"
        :description="detail.description"
        description-fallback="暂无作品简介。"
        :on-bookshelf="onBookshelf"
        :loading="loading"
        :resume-chapter-id="resumeChapterId"
        :can-start-reading="chapterCount > 0"
        :has-chapters="chapterCount > 0"
        :stats="stats"
        section-title="作品目录"
        :section-summary="`共 ${chapterCount} 话`"
        @toggle-bookshelf="toggleBookshelf"
        @continue-reading="continueReading"
      >
        <BookChapterList
          :items="chapterItems"
          :loading="loading"
          @open="openChapter($event.data.id)"
        />
      </BookDetailLayout>
    </section>
  </div>
</template>
