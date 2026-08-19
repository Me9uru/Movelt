<script setup lang="ts">
import { Collection, Search } from "@element-plus/icons-vue";
import { nextTick, ref } from "vue";
import type { BookshelfEntry } from "../../services/library";
import type { NovelSummary } from "../../services/novel";
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
  openNovel: [novel: NovelSummary];
  search: [];
  "update:query": [value: string];
}>();

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

</script>

<template>
  <section class="bookshelf-view">
    <section class="bookshelf-section">
      <div class="section-heading">
        <div class="bookshelf-heading-actions">
          <el-tag class="count-tag" effect="plain">
            {{ searchActive ? `${books.length} / ${totalBooks} 本` : `${totalBooks} 本` }}
          </el-tag>
          <el-button
            class="library-search-trigger"
            :icon="Search"
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
            :src="entry.book.cover_url"
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
            <span>LightNovelShelf</span>
          </div>
        </el-card>
      </div>

      <el-empty
        v-if="!bookshelfLoading && searchActive && books.length === 0"
        :image-size="112"
        description="没有找到匹配的书籍"
      />

      <el-empty v-else-if="!bookshelfLoading && books.length === 0" :image-size="112" description="书架还是空的">
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
      :with-header="false"
      :show-close="false"
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
