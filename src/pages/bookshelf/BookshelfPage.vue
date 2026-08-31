<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import type {
  ComicBookshelfEntry,
  NovelBookshelfEntry,
} from "../../domain/bookshelf";
import type { NovelSummary } from "../../domain/novel";
import type { ComicSummary } from "../../domain/comic";
import ContentTabs from "../../layout/ContentTabs.vue";
import BookCollection from "../../layout/BookCollection.vue";
import type { BookGridItem } from "../../types/book";
import { useLibrary } from "../../composables/useLibrary";
import { showError } from "../../utils/error";

const router = useRouter();
const {
  books,
  comic,
  refreshBooks,
  refreshComicBooks,
  searchBooks,
  searchComicBooks,
} = useLibrary();
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

async function load(): Promise<void> {
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

async function search(): Promise<void> {
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

function changeKind(kind: "novel" | "comic"): void {
  activeKind.value = kind;
  query.value = "";
  searchResults.value = null;
  comicSearchResults.value = null;
  void load();
}

function openNovel(novel: NovelSummary): void {
  void router.push({
    name: "detail",
    params: { bookId: novel.id },
    query: { from: "bookshelf" },
  });
}
function openComic(item: ComicSummary): void {
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
    <ContentTabs
      :model-value="activeKind"
      :tabs="shelfTabs"
      search-label="搜索书架"
      :query="query"
      :search-loading="loading"
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
