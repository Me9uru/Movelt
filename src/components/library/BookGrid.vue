<script setup lang="ts">
import type { NovelSummary } from "../../services/novel";
import NovelCover from "./NovelCover.vue";

defineProps<{ books: NovelSummary[]; loading?: boolean }>();
const emit = defineEmits<{ openNovel: [novel: NovelSummary] }>();
</script>

<template>
  <div v-if="loading" class="result-grid" aria-label="正在加载">
    <el-card v-for="item in 6" :key="item" shadow="never" class="book-card skeleton-card">
      <el-skeleton animated>
        <template #template>
          <el-skeleton-item variant="image" class="skeleton-cover" />
          <el-skeleton-item variant="h3" style="width: 82%" />
          <el-skeleton-item variant="text" style="width: 55%" />
        </template>
      </el-skeleton>
    </el-card>
  </div>
  <div v-else class="result-grid">
    <el-card
      v-for="novel in books"
      :key="`${novel.source}:${novel.id}`"
      class="book-card"
      shadow="hover"
      tabindex="0"
      @click="emit('openNovel', novel)"
      @keydown.enter="emit('openNovel', novel)"
    >
      <NovelCover
        class="book-cover"
        :source="novel.source"
        :novel-id="novel.id"
        :title="novel.title"
        :cover-url="novel.cover_url"
      />
      <div class="book-meta">
        <strong>{{ novel.title }}</strong>
        <span v-if="novel.author || novel.status">{{ novel.author || novel.status }}</span>
        <div v-if="novel.tags.length" class="book-tags">
          <el-tag v-for="tag in novel.tags.slice(0, 2)" :key="tag" size="small" effect="plain">{{ tag }}</el-tag>
        </div>
      </div>
    </el-card>
  </div>
</template>
