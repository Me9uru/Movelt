<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import type { ChapterSummary, NovelSummary, Volume } from "../../domain/novel";
import BookChapterList from "../../components/detail/BookChapterList.vue";
import type { BookChapterItem } from "../../types/book";
import BookDetailLayout from "../../layout/BookDetailLayout.vue";
import LoadingOverlay from "../../components/common/LoadingOverlay.vue";
import ErrorState from "../../components/common/ErrorState.vue";
import { useLibrary } from "../../composables/useLibrary";
import { getReaderOverview, lightNovelSourceId } from "../../services/novel";
import { getErrorMessage, showError } from "../../utils/error";

const route = useRoute();
const router = useRouter();
const { addBook, removeBook, isOnBookshelf } = useLibrary();
const detail = ref<NovelSummary | null>(null);
const catalogue = ref<Volume[]>([]);
const resumeChapterId = ref<string | null>(null);
const loading = ref(true);
const error = ref("");
const bookId = computed(() => typeof route.params.bookId === "string" ? route.params.bookId : "");
const from = computed(() => route.query.from === "bookshelf" || route.query.from === "novel-search" ? route.query.from : "novels");
const backLabel = computed(() => `返回${from.value === "bookshelf" ? "书架" : "小说"}`);
const onBookshelf = computed(() => detail.value ? isOnBookshelf(detail.value) : false);

async function load(): Promise<void> {
  if (!bookId.value) return;
  loading.value = true;
  error.value = "";
  try {
    const overview = await getReaderOverview(lightNovelSourceId, bookId.value);
    detail.value = overview.detail;
    catalogue.value = overview.volumes;
    resumeChapterId.value = overview.readPosition?.chapterId ?? null;
  } catch (value) {
    error.value = getErrorMessage(value, "无法加载作品详情");
    showError(value, "无法加载作品详情");
  } finally {
    loading.value = false;
  }
}

function goBack(): void {
  if (window.history.state?.back) router.back();
  else void router.replace({ name: from.value });
}

function openChapter(chapterId: string): void {
  if (!detail.value) return;
  void router.push({ name: "reader", params: { bookId: detail.value.id, chapterId }, query: route.query });
}

function continueReading(): void {
  const chapterId = resumeChapterId.value ?? catalogue.value.flatMap(collectChapterItems)[0]?.data.id;
  if (chapterId) openChapter(chapterId);
}

async function toggleBookshelf(): Promise<void> {
  if (!detail.value) return;
  loading.value = true;
  try {
    if (onBookshelf.value) await removeBook(detail.value);
    else await addBook(detail.value);
  } catch (value) {
    showError(value, "更新书架失败");
  } finally {
    loading.value = false;
  }
}

onMounted(() => void load());
watch(bookId, () => void load());

const chapterCount = computed(() =>
  catalogue.value.reduce((total, volume) => total + countChapters(volume), 0),
);
const stats = computed(() => [
  { value: catalogue.value.length, label: "篇" },
  { value: chapterCount.value, label: "话" },
  ...(detail.value?.updated_at ? [{ label: `更新于 ${detail.value.updated_at}` }] : []),
]);

function countChapters(volume: Volume): number {
  return volume.chapters.length + volume.sections.reduce((total, section) => total + countChapters(section), 0);
}

function toChapterItem(chapter: ChapterSummary): BookChapterItem<ChapterSummary> {
  return { id: chapter.id, title: chapter.title, data: chapter };
}

function collectChapterItems(volume: Volume): BookChapterItem<ChapterSummary>[] {
  return [
    ...volume.chapters.map(toChapterItem),
    ...volume.sections.flatMap(collectChapterItems),
  ];
}

const chapterItems = computed(() => catalogue.value.flatMap(collectChapterItems));
</script>

<template>
  <div>
    <header class="topbar book-detail-topbar">
      <div class="topbar-inner detail-topbar">
        <var-button text @click="goBack"><var-icon name="arrow-left" />{{ backLabel }}</var-button>
      </div>
    </header>
    <section class="detail-view">
    <LoadingOverlay v-if="loading && !detail" inline visible label="正在加载作品详情" />
    <ErrorState v-else-if="error" title="作品详情加载失败" :message="error" :loading="loading" @retry="load" />
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
    :stats="stats"
    section-title="作品目录"
    :section-summary="`共 ${catalogue.length} 篇 · ${chapterCount} 话`"
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
