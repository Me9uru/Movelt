<script setup lang="ts">
import type { MangaChapter } from "../../domain/content";

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
    <var-icon name="book-open-variant" />
    <span>
      <strong>{{ chapter.name || `第 ${chapter.chapterNumber} 话` }}</strong>
      <small v-if="chapter.pageCount">{{ chapter.pageCount }} 页</small>
    </span>
    <var-chip v-if="!chapter.isRead" size="small">未读</var-chip>
  </button>
</template>
