<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRouter } from "vue-router";
import type {
  ComicBookshelfEntry,
  NovelBookshelfEntry,
} from "../../domain/bookshelf";
import type { NovelSummary } from "../../domain/novel";
import type { ComicSummary } from "../../domain/comic";
import BookshelfTabs from "../../components/bookshelf/BookshelfTabs.vue";
import BookCollection from "../../layout/BookCollection.vue";
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
const activeKind = ref<"novel" | "comic">("novel");
const query = ref("");
const searchResults = ref<NovelBookshelfEntry[] | null>(null);
const comicSearchResults = ref<ComicBookshelfEntry[] | null>(null);
const loading = ref(false);
const bookshelfLoading = ref(false);
const visibleBooks = computed(() => searchResults.value ?? books.value);
const visibleComic = computed(() => comicSearchResults.value ?? comic.value);
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
const searchActive = computed(
  () => searchResults.value !== null || comicSearchResults.value !== null,
);

const shelfTabs: { name: "novel" | "comic"; label: string }[] = [
  { name: "novel", label: "小说" },
  { name: "comic", label: "漫画" },
];

const load = async (): Promise<void> => {
  bookshelfLoading.value = true;
  try {
    if (activeKind.value === "novel") await refreshBooks();
    else await refreshComicBooks();
  } catch (error) {
    showError(error, "加载书架失败");
  } finally {
    bookshelfLoading.value = false;
  }
}

const search = async (): Promise<void> => {
  const value = query.value.trim();
  if (!value) {
    if (activeKind.value === "novel") searchResults.value = null;
    else comicSearchResults.value = null;
    return;
  }
  loading.value = true;
  try {
    if (activeKind.value === "novel") searchResults.value = await searchBooks(value);
    else comicSearchResults.value = searchComicBooks(value);
  } catch (error) {
    showError(error, "搜索书架失败");
  } finally {
    loading.value = false;
  }
}

const changeKind = (kind: "novel" | "comic"): void => {
  activeKind.value = kind;
  query.value = "";
  searchResults.value = null;
  comicSearchResults.value = null;
  void load();
}

const openNovel = (novel: NovelSummary): void => {
  void router.push({
    name: "detail",
    params: { bookId: novel.id },
    query: { from: "bookshelf" },
  });
}
const openComic = (item: ComicSummary): void => {
  void router.push({ name: "comic-detail", params: { comicId: item.id } });
}

onMounted(() => void load());
watch(query, (value) => {
  if (!value.trim()) {
    searchResults.value = null;
    comicSearchResults.value = null;
  }
});
</script>

<template>
  <section class="bookshelf-view">
    <BookshelfTabs
      :model-value="activeKind"
      :tabs="shelfTabs"
      :query="query"
      :loading="loading"
      @update:model-value="changeKind($event)"
      @search="search"
      @clear="search"
      @update:query="query = $event"
    />
    <BookCollection
      v-if="activeKind === 'novel'"
      :items="novelGridItems"
      :loading="bookshelfLoading"
      :disabled="loading || bookshelfLoading"
      @open="openNovel($event.data)"
    >
      <template #empty>
        <div class="bookshelf-empty">
          <div class="bookshelf-empty__icon" aria-hidden="true">
            <var-icon :name="searchActive ? 'magnify' : 'bookmark'" />
          </div>
          <h2>
            {{ searchActive ? "没有找到匹配的书籍" : "书架还是空的" }}
          </h2>
          <p>
            {{
              searchActive
                ? "换个关键词试试看。"
                : "把想看的作品收藏起来，方便下次继续阅读。"
            }}
          </p>
        </div>
      </template>
    </BookCollection>
    <BookCollection
      v-else
      :items="comicGridItems"
      :loading="bookshelfLoading"
      :disabled="loading || bookshelfLoading"
      @open="openComic($event.data)"
    >
      <template #empty>
        <div class="bookshelf-empty">
          <div class="bookshelf-empty__icon" aria-hidden="true">
            <var-icon :name="searchActive ? 'magnify' : 'bookmark'" />
          </div>
          <h2>
            {{ searchActive ? "没有找到匹配的书籍" : "漫画收藏还是空的" }}
          </h2>
          <p>
            {{
              searchActive
                ? "换个关键词试试看。"
                : "把想看的作品收藏起来，方便下次继续阅读。"
            }}
          </p>
        </div>
      </template>
    </BookCollection>
  </section>
</template>
