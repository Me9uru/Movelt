<script setup lang="ts">
import type { BookshelfEntry } from "../../services/bookshelf";
import type { MangaSummary, NovelSummary } from "../../domain/content";
import ContentTabs from "../../components/common/ContentTabs.vue";
import LoadingOverlay from "../../components/common/LoadingOverlay.vue";
import WorkCover from "../../components/common/WorkCover.vue";

const props = defineProps<{
  books: BookshelfEntry[];
  manga: MangaSummary[];
  activeKind: "novel" | "manga";
  loading: boolean;
  bookshelfLoading: boolean;
  query: string;
  searchActive: boolean;
}>();

const emit = defineEmits<{
  browse: [];
  openNovel: [novel: NovelSummary];
  openManga: [manga: MangaSummary];
  search: [];
  "update:activeKind": [value: "novel" | "manga"];
  "update:query": [value: string];
}>();

const shelfTabs: { name: "novel" | "manga"; label: string }[] = [
  { name: "novel", label: "小说" },
  { name: "manga", label: "漫画" },
];

</script>

<template>
  <section class="bookshelf-view">
    <ContentTabs
      :model-value="activeKind"
      :tabs="shelfTabs"
      search-label="搜索书架"
      :query="query"
      :search-loading="loading"
      @update:model-value="emit('update:activeKind', $event)"
      @search="emit('search')"
      @clear="emit('search')"
      @update:query="emit('update:query', $event)"
    />
    <LoadingOverlay
      v-if="bookshelfLoading"
      inline
      visible
      label="正在加载书架"
    />
    <div v-else class="result-grid">
      <var-card
        v-if="activeKind === 'novel'"
        v-for="entry in books"
        :key="`${entry.book.source}:${entry.book.id}`"
        class="book-card shelf-card"
        :class="{ 'book-card--disabled': loading }"
        hoverable
        :tabindex="loading ? -1 : 0"
        :aria-disabled="loading"
        @click="emit('openNovel', entry.book)"
        @keydown.enter="emit('openNovel', entry.book)"
      >
        <WorkCover
          class="book-cover"
          :title="entry.book.title"
          :cover-url="entry.book.cover_url"
        />
        <div class="book-meta">
          <strong>{{ entry.book.title }}</strong>
        </div>
      </var-card>
      <var-card
        v-else
        v-for="item in manga"
        :key="item.id"
        class="book-card shelf-card"
        :class="{ 'book-card--disabled': loading }"
        hoverable
        :tabindex="loading ? -1 : 0"
        :aria-disabled="loading"
        @click="emit('openManga', item)"
        @keydown.enter="emit('openManga', item)"
      >
        <WorkCover class="book-cover" :cover-url="item.thumbnailUrl" :title="item.title" />
        <div class="book-meta"><strong>{{ item.title }}</strong><span v-if="item.author">{{ item.author }}</span></div>
      </var-card>
    </div>

    <div
      v-if="activeKind === 'novel' && !bookshelfLoading && searchActive && books.length === 0"
      class="bookshelf-empty"
    >
      <div class="bookshelf-empty__icon" aria-hidden="true">
        <var-icon name="magnify" />
      </div>
      <h2>没有找到匹配的书籍</h2>
      <p>换个关键词试试看。</p>
    </div>

    <div
      v-else-if="!bookshelfLoading && (activeKind === 'novel' ? books.length === 0 : manga.length === 0)"
      class="bookshelf-empty"
    >
      <div class="bookshelf-empty__icon" aria-hidden="true">
        <var-icon name="bookmark" />
      </div>
      <h2>{{ activeKind === "novel" ? "书架还是空的" : "漫画收藏还是空的" }}</h2>
      <p>把想看的作品收藏起来，方便下次继续阅读。</p>
      <var-button type="primary" outline @click="emit('browse')">
        {{ activeKind === "novel" ? "去找小说" : "去找漫画" }}
      </var-button>
    </div>

  </section>
</template>
