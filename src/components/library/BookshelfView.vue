<script setup lang="ts">
import { Collection } from "@element-plus/icons-vue";
import type { BookshelfEntry } from "../../domain/library";
import type { NovelSummary } from "../../services/novel";
import ReadingProgressBar from "./ReadingProgressBar.vue";

defineProps<{
  books: BookshelfEntry[];
  loading: boolean;
  bookshelfLoading: boolean;
}>();

const emit = defineEmits<{
  browse: [];
  openNovel: [novel: NovelSummary];
}>();

</script>

<template>
  <section class="bookshelf-view">
    <section class="bookshelf-section">
      <div class="section-heading">
        <div>
          <h2>我的书架</h2>
        </div>
        <el-tag round effect="plain">{{ books.length }} 本</el-tag>
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
            lazy
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
      </div>

      <el-empty v-if="!bookshelfLoading && books.length === 0" :image-size="112" description="书架还是空的">
        <el-button type="primary" plain @click="emit('browse')">
          去找小说
        </el-button>
      </el-empty>
    </section>
  </section>
</template>
