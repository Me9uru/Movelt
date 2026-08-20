<script setup lang="ts">
import { Collection, Picture, Search } from "@element-plus/icons-vue";
import { nextTick, ref } from "vue";
import type { BookshelfEntry } from "../../services/library";
import type { MangaSummary } from "../../services/manga";
import type { NovelSummary } from "../../services/novel";
import BookSearchBar from "../../components/common/BookSearchBar.vue";
import LoadingOverlay from "../../components/common/LoadingOverlay.vue";

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

const searchDialogVisible = ref(false);

function beforeTabLeave(nextName: string | number): boolean {
  if (nextName !== "search") return true;

  searchDialogVisible.value = true;
  return false;
}

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
      <el-tabs
        :model-value="activeKind"
        class="discovery-tabs"
        :before-leave="beforeTabLeave"
        @update:model-value="emit('update:activeKind', $event as 'novel' | 'manga')"
      >
        <el-tab-pane label="小说" name="novel" />
        <el-tab-pane label="漫画" name="manga" />
        <el-tab-pane name="search">
          <template #label>
            <el-icon class="library-search-trigger discovery-search-trigger" aria-label="搜索书架"><Search /></el-icon>
          </template>
        </el-tab-pane>
      </el-tabs>
      <LoadingOverlay
        v-if="bookshelfLoading"
        inline
        visible
        label="正在加载书架"
      />
      <div v-else class="result-grid">
        <el-card
          v-if="activeKind === 'novel'"
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
          </div>
        </el-card>
        <el-card
          v-else
          v-for="item in manga"
          :key="item.id"
          class="book-card shelf-card"
          :class="{ 'book-card--disabled': loading }"
          shadow="hover"
          :tabindex="loading ? -1 : 0"
          :aria-disabled="loading"
          @click="emit('openManga', item)"
          @keydown.enter="emit('openManga', item)"
        >
          <el-image v-if="item.thumbnailUrl" class="book-cover" :src="item.thumbnailUrl" :alt="item.title" fit="cover">
            <template #error><div class="cover-placeholder"><el-icon><Picture /></el-icon></div></template>
          </el-image>
          <div v-else class="cover-placeholder"><el-icon><Picture /></el-icon></div>
          <div class="book-meta"><strong>{{ item.title }}</strong><span v-if="item.author">{{ item.author }}</span></div>
        </el-card>
      </div>

      <el-empty
        v-if="activeKind === 'novel' && !bookshelfLoading && searchActive && books.length === 0"
        :image-size="112"
        description="没有找到匹配的书籍"
      />

      <el-empty v-else-if="!bookshelfLoading && (activeKind === 'novel' ? books.length === 0 : manga.length === 0)" :image-size="112" :description="activeKind === 'novel' ? '书架还是空的' : '漫画收藏还是空的'">
        <el-button type="primary" plain @click="emit('browse')">
          {{ activeKind === "novel" ? "去找小说" : "去找漫画" }}
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
