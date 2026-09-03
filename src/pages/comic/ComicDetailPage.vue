<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  getComicSeries,
  isOnComicBookshelf,
} from "../../services/comic";
import { useBookshelfStore } from "../../stores/bookshelf";
import type {
  ComicBook,
  ComicChapterSummary,
  ComicSeriesDetail,
} from "../../domain/comic";
import { getErrorMessage, showError } from "../../utils/error";
import BookChapterList from "../../components/book/BookChapterList.vue";
import type { BookChapterGroup, BookChapterItem } from "../../types/book";
import BookDetailLayout from "../../layout/BookDetailLayout.vue";
import ErrorState from "../../components/common/ErrorState.vue";
import LoadingOverlay from "../../components/common/LoadingOverlay.vue";

const route = useRoute();
const router = useRouter();
const bookshelf = useBookshelfStore();
const comic = ref<ComicSeriesDetail | null>(null);
const loading = ref(true);
const error = ref("");
const comicId = computed(() => String(route.params.comicId));
const onBookshelf = ref(false);
interface ComicChapterTarget {
  bookId: string;
  chapter: ComicChapterSummary;
}

const resumeTarget = computed<ComicChapterTarget | null>(() => {
  for (const book of comic.value?.books ?? []) {
    const chapterId = book.readPosition?.chapterId;
    const chapter = book.chapters.find((item) => item.id === chapterId);
    if (chapter) return { bookId: book.id, chapter };
  }
  return null;
});

const toChapterItem = (
  book: ComicBook,
  chapter: ComicChapterSummary,
): BookChapterItem<ComicChapterTarget> => {
  return {
    id: chapter.id,
    title: chapter.title || `第 ${chapter.sequence} 话`,
    icon: "book-open-variant",
    meta: chapter.pageCount > 0 ? `${chapter.pageCount} 页` : undefined,
    data: { bookId: book.id, chapter },
  };
}

const chapterGroups = computed<BookChapterGroup<ComicChapterTarget>[]>(() =>
  (comic.value?.books ?? []).map((book) => ({
    title: book.title,
    count: book.chapters.length,
    chapters: book.chapters.map((chapter) => toChapterItem(book, chapter)),
  })),
);
const chapterCount = computed(() =>
  (comic.value?.books ?? []).reduce(
    (count, book) => count + book.chapters.length,
    0,
  ),
);
const firstTarget = computed<ComicChapterTarget | null>(() => {
  const book = comic.value?.books[0];
  const chapter = book?.chapters[0];
  return book && chapter ? { bookId: book.id, chapter } : null;
});

const read = (bookId: string, chapterId: string): void => {
  void router.push({ name: "comic-reader", params: { comicId: bookId, chapterId } });
}

const load = async (): Promise<void> => {
  loading.value = true;
  error.value = "";
  try {
    const detail = await getComicSeries(comicId.value);
    comic.value = detail;
    onBookshelf.value = detail.books[0]
      ? await isOnComicBookshelf(detail.books[0].id)
      : false;
  } catch (value) {
    error.value = getErrorMessage(value, "无法加载漫画详情");
    showError(value, "无法加载漫画详情");
  } finally {
    loading.value = false;
  }
}
onMounted(() => void load());
const continueReading = (): void => {
  const target = resumeTarget.value ?? firstTarget.value;
  if (target) read(target.bookId, target.chapter.id);
}
const goBack = (): void => {
  if (window.history.state?.back) router.back();
  else void router.replace({
    name: "comic",
    query: {
      tab: route.query.tab,
      q: route.query.q,
      mode: route.query.mode,
    },
  });
}
const toggleBookshelf = async (): Promise<void> => {
  if (!comic.value) return;
  try {
    const book = comic.value.books[0];
    if (!book) return;
    const shelfItem = { ...comic.value, id: book.id };
    if (onBookshelf.value) await bookshelf.removeComicBook(book.id);
    else await bookshelf.addComicBook(shelfItem);
    onBookshelf.value = !onBookshelf.value;
  } catch (value) {
    showError(value, "更新漫画书架失败");
  }
}
</script>
<template>
  <div>
    <header class="topbar book-detail-topbar">
      <div class="topbar-inner detail-topbar">
        <var-button text @click="goBack"><var-icon name="arrow-left" />返回漫画</var-button>
      </div>
    </header>
    <section class="detail-view">
      <LoadingOverlay v-if="loading" inline visible label="正在加载漫画详情" />
      <ErrorState v-if="error" title="漫画详情加载失败" :message="error" :loading="loading" @retry="load" />
  <BookDetailLayout
    v-else-if="comic"
    :title="comic?.title ?? ''"
    :cover-url="comic?.coverUrl ?? null"
    :author="comic?.author || '作者未知'"
    :status="comic?.status"
    :tags="comic?.genre ?? []"
    :description="comic?.description"
    :on-bookshelf="onBookshelf"
    :loading="loading"
    :resume-chapter-id="resumeTarget?.chapter.id"
    :can-start-reading="Boolean(firstTarget)"
    :stats="[
      { value: comic.books.length, label: '卷' },
      { value: chapterCount, label: '话' },
    ]"
    section-title="分卷与章节"
    :section-summary="`共 ${comic.books.length} 卷 · ${chapterCount} 话`"
    @toggle-bookshelf="toggleBookshelf"
    @continue-reading="continueReading"
  >
    <BookChapterList
      :groups="chapterGroups"
      @open="read($event.data.bookId, $event.data.chapter.id)"
    />
  </BookDetailLayout>
    </section>
  </div>
</template>
