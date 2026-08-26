<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import type { BookshelfEntry } from "../../domain/bookshelf";
import type { NovelSummary } from "../../domain/novel";
import type { MangaSummary } from "../../domain/manga";
import ContentTabs from "../../layout/ContentTabs.vue";
import BookCollection from "../../layout/BookCollection.vue";
import type { BookGridItem } from "../../types/book";
import { useLibrary } from "../../composables/useLibrary";
import { listMangaBookshelf } from "../../services/manga";
import { showError } from "../../utils/error";

const router = useRouter();
const { books, refreshBooks, searchBooks } = useLibrary();
const manga = ref<MangaSummary[]>([]);
const activeKind = ref<"novel" | "manga">("novel");
const query = ref("");
const searchResults = ref<BookshelfEntry[] | null>(null);
const mangaSearchResults = ref<MangaSummary[] | null>(null);
const loading = ref(false);
const bookshelfLoading = ref(false);
const visibleBooks = computed(() => searchResults.value ?? books.value);
const visibleManga = computed(() => mangaSearchResults.value ?? manga.value);
const novelGridItems = computed<BookGridItem<NovelSummary>[]>(() =>
  visibleBooks.value.map(({ book }) => ({
    id: `${book.source}:${book.id}`,
    title: book.title,
    coverUrl: book.cover_url,
    data: book,
  })),
);
const mangaGridItems = computed<BookGridItem<MangaSummary>[]>(() =>
  visibleManga.value.map((item) => ({
    id: item.id,
    title: item.title,
    coverUrl: item.thumbnailUrl,
    meta: item.author,
    data: item,
  })),
);
const searchActive = computed(
  () => searchResults.value !== null || mangaSearchResults.value !== null,
);

const shelfTabs: { name: "novel" | "manga"; label: string }[] = [
  { name: "novel", label: "小说" },
  { name: "manga", label: "漫画" },
];

async function load(): Promise<void> {
  bookshelfLoading.value = true;
  try {
    if (activeKind.value === "novel") await refreshBooks();
    else manga.value = await listMangaBookshelf();
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
    else mangaSearchResults.value = null;
    return;
  }
  loading.value = true;
  try {
    if (activeKind.value === "novel") searchResults.value = await searchBooks(value);
    else mangaSearchResults.value = await listMangaBookshelf(value);
  } catch (error) {
    showError(error, "搜索书架失败");
  } finally {
    loading.value = false;
  }
}

function changeKind(kind: "novel" | "manga"): void {
  activeKind.value = kind;
  query.value = "";
  searchResults.value = null;
  mangaSearchResults.value = null;
  void load();
}

function openNovel(novel: NovelSummary): void {
  void router.push({
    name: "detail",
    params: { bookId: novel.id },
    query: { from: "bookshelf" },
  });
}
function openManga(item: MangaSummary): void {
  void router.push({ name: "manga-detail", params: { mangaId: item.id } });
}

onMounted(() => void load());
watch(query, (value) => {
  if (!value.trim()) {
    searchResults.value = null;
    mangaSearchResults.value = null;
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
      :items="mangaGridItems"
      :loading="bookshelfLoading"
      :disabled="loading || bookshelfLoading"
      @open="openManga($event.data)"
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
