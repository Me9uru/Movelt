<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { storeToRefs } from "pinia";
import { useRouter } from "vue-router";
import type { NovelSummary } from "../../domain/novel";
import type { ComicSummary } from "../../domain/comic";
import BookSearchBar from "../../components/book/BookSearchBar.vue";
import BookshelfCollection from "../../components/bookshelf/BookshelfCollection.vue";
import TabbedPageLayout from "../../layout/TabbedPageLayout.vue";
import type { BookGridItem } from "../../types/book";
import { useBookshelfStore } from "../../stores/bookshelf";
import { showError } from "../../utils/error";

const router = useRouter();
const bookshelf = useBookshelfStore();
const { books, comic } = storeToRefs(bookshelf);
const {
  refreshBooks,
  refreshComicBooks,
  searchBooks,
  searchComicBooks,
} = bookshelf;
type BookshelfKind = "novel" | "comic";

const activeKind = ref<BookshelfKind>("novel");
const query = ref("");
const bookshelfLoading = ref(false);
const normalizedQuery = computed(() => query.value.trim());
const filtering = computed(() => normalizedQuery.value.length > 0);
const visibleBooks = computed(() =>
  filtering.value ? searchBooks(normalizedQuery.value) : books.value,
);
const visibleComic = computed(() =>
  filtering.value ? searchComicBooks(normalizedQuery.value) : comic.value,
);
const novelGridItems = computed<BookGridItem<NovelSummary>[]>(() =>
  visibleBooks.value.map(({ book, progress }) => ({
    id: `${book.source}:${book.id}`,
    title: book.title,
    coverUrl: book.cover_url,
    coverStatus: progress === 100 ? "finished" : "unfinished",
    data: book,
  })),
);
const comicGridItems = computed<BookGridItem<ComicSummary>[]>(() =>
  visibleComic.value.map(({ comic, progress }) => ({
    id: comic.id,
    title: comic.title,
    coverUrl: comic.coverUrl,
    meta: comic.author,
    coverStatus: progress === 100 ? "finished" : "unfinished",
    data: comic,
  })),
);
const shelfTabs: { name: BookshelfKind; label: string }[] = [
  { name: "novel", label: "小说" },
  { name: "comic", label: "漫画" },
];

const load = async (kind: BookshelfKind): Promise<void> => {
  bookshelfLoading.value = true;
  try {
    if (kind === "novel") await refreshBooks();
    else await refreshComicBooks();
  } catch (error) {
    showError(error, "加载书架失败");
  } finally {
    bookshelfLoading.value = false;
  }
};

const changeKind = (kind: BookshelfKind): void => {
  activeKind.value = kind;
  query.value = "";
  void load(kind);
};

const openNovel = (novel: NovelSummary): void => {
  void router.push({
    name: "novel-detail",
    params: { bookId: novel.id },
    query: { from: "bookshelf" },
  });
};
const openComic = (item: ComicSummary): void => {
  void router.push({ name: "comic-detail", params: { comicId: item.id } });
};

onMounted(() => void load("novel"));
</script>

<template>
  <TabbedPageLayout
    :model-value="activeKind"
    :tabs="shelfTabs"
    @update:model-value="changeKind"
  >
    <BookSearchBar
      class="bookshelf-filter"
      :model-value="query"
      :loading="bookshelfLoading"
      :show-submit="false"
      :subject-label="activeKind === 'novel' ? '小说' : '漫画'"
      @update:model-value="query = $event"
      @clear="query = ''"
    />
    <BookshelfCollection
      v-if="activeKind === 'novel'"
      :items="novelGridItems"
      :loading="bookshelfLoading"
      :filtering="filtering"
      empty-title="书架还是空的"
      @open="openNovel"
    />
    <BookshelfCollection
      v-else
      :items="comicGridItems"
      :loading="bookshelfLoading"
      :filtering="filtering"
      empty-title="漫画收藏还是空的"
      @open="openComic"
    />
  </TabbedPageLayout>
</template>
