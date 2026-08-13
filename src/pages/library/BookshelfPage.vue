<script setup lang="ts">
import { Collection, Plus, Search } from "@element-plus/icons-vue";
import { nextTick, ref, watch } from "vue";
import type { BookshelfEntry } from "../../domain/library";
import type { NovelSummary } from "../../services/novel";
import {
  canUseLocalEpubAssets,
  localEpubAssetUrl,
  localEpubSourceId,
} from "../../services/localEpub";
import ReadingProgressBar from "../../components/library/ReadingProgressBar.vue";
import BookSearchBar from "../../components/common/BookSearchBar.vue";

const props = defineProps<{
  books: BookshelfEntry[];
  loading: boolean;
  bookshelfLoading: boolean;
  query: string;
  totalBooks: number;
  searchActive: boolean;
}>();

const emit = defineEmits<{
  browse: [];
  importEpub: [];
  openNovel: [novel: NovelSummary];
  search: [];
  "update:query": [value: string];
}>();

const canImportEpub = canUseLocalEpubAssets();
const localCoverUrls = ref<Record<string, string | null>>({});
const searchDialogVisible = ref(false);

function focusSearchInput() {
  const input = document.querySelector<HTMLInputElement>(".bookshelf-search-dialog input");
  input?.focus();
}

function submitSearch() {
  if (!props.query.trim()) {
    void nextTick(focusSearchInput);
    return;
  }

  emit("search");
  searchDialogVisible.value = false;
}

watch(
  () => props.books,
  async (books) => {
    const covers = await Promise.all(books
      .filter((entry) => entry.book.source === localEpubSourceId && entry.book.cover_url)
      .map(async (entry) => [
        entry.book.id,
        await localEpubAssetUrl(entry.book.id, entry.book.cover_url),
      ] as const));
    localCoverUrls.value = Object.fromEntries(covers);
  },
  { immediate: true },
);

</script>

<template>
  <section class="bookshelf-view">
    <section class="bookshelf-section">
      <div class="section-heading">
        <div>
          <h2>我的书架</h2>
        </div>
        <div class="bookshelf-heading-actions">
          <el-tag effect="plain">
            {{ searchActive ? `${books.length} / ${totalBooks} 本` : `${totalBooks} 本` }}
          </el-tag>
          <el-button
            class="bookshelf-search-trigger"
            :icon="Search"
            circle
            aria-label="搜索书架"
            title="搜索书架"
            @click="searchDialogVisible = true"
          />
        </div>
      </div>

      <div class="result-grid">
        <el-card
          v-for="entry in books"
          :key="`${entry.book.source}:${entry.book.id}`"
          class="book-card shelf-card"
          :class="{ 'book-card--disabled': loading }"
          shadow="hover"
          :tabindex="loading ? -1 : 0"
          :aria-disabled="loading"
          @click="emit('openNovel', entry.book)"
          @keydown.enter="emit('openNovel', entry.book)"
        >
          <el-image
            v-if="entry.book.cover_url"
            class="book-cover"
            :src="entry.book.source === localEpubSourceId ? localCoverUrls[entry.book.id] ?? undefined : entry.book.cover_url"
            :alt="entry.book.title"
            fit="cover"
          >
            <template #error>
              <div class="cover-placeholder">
                <el-icon><Collection /></el-icon>
              </div>
            </template>
          </el-image>
          <div v-else class="cover-placeholder">
            <el-icon><Collection /></el-icon>
          </div>
          <div class="book-meta">
            <strong>{{ entry.book.title }}</strong>
            <ReadingProgressBar v-if="entry.progress" :progress="entry.progress" />
            <span v-else>尚未开始阅读</span>
          </div>
        </el-card>
        <el-card
          v-if="canImportEpub"
          class="book-card shelf-card import-epub-card"
          :class="{ 'book-card--disabled': loading }"
          shadow="hover"
          :tabindex="loading ? -1 : 0"
          :aria-disabled="loading"
          role="button"
          aria-label="导入 EPUB"
          @click="emit('importEpub')"
          @keydown.enter="emit('importEpub')"
          @keydown.space.prevent="emit('importEpub')"
        >
          <div class="cover-placeholder import-epub-cover" aria-hidden="true">
            <el-icon><Collection /></el-icon>
            <el-icon class="import-epub-plus"><Plus /></el-icon>
          </div>
          <div class="book-meta import-epub-meta">
            <strong>导入 EPUB</strong>
            <span>添加本地书籍</span>
          </div>
        </el-card>
      </div>

      <el-empty
        v-if="!bookshelfLoading && searchActive && books.length === 0"
        :image-size="112"
        description="没有找到匹配的书籍"
      />

      <el-empty v-else-if="!bookshelfLoading && books.length === 0 && !canImportEpub" :image-size="112" description="书架还是空的">
        <el-button type="primary" plain @click="emit('browse')">
          去找小说
        </el-button>
      </el-empty>
    </section>

    <el-dialog
      v-model="searchDialogVisible"
      class="library-search-dialog bookshelf-search-dialog"
      modal-class="library-search-mask"
      width="min(560px, calc(100vw - 32px))"
      align-center
      destroy-on-close
      title="搜索书架"
      @opened="focusSearchInput"
    >
      <BookSearchBar
        class="bookshelf-search"
        :model-value="query"
        :loading="loading"
        @update:model-value="emit('update:query', $event)"
        @clear="emit('search')"
        @submit="submitSearch"
      />
    </el-dialog>
  </section>
</template>
