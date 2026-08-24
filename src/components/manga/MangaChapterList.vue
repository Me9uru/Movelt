<script setup lang="ts">
import { Reading } from "@element-plus/icons-vue";
import type { MangaChapter } from "../../services/manga";

defineProps<{
  chapters: MangaChapter[];
}>();

const emit = defineEmits<{
  openChapter: [chapterId: string];
}>();
</script>

<template>
  <button
    v-for="chapter in chapters"
    :key="chapter.id"
    class="manga-chapter"
    type="button"
    @click="emit('openChapter', chapter.id)"
  >
    <el-icon><Reading /></el-icon>
    <span>
      <strong>{{ chapter.name || `第 ${chapter.chapterNumber} 话` }}</strong>
      <small v-if="chapter.pageCount">{{ chapter.pageCount }} 页</small>
    </span>
    <el-tag v-if="!chapter.isRead" size="small">未读</el-tag>
  </button>
</template>
