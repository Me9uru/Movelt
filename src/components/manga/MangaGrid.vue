<script setup lang="ts">
import type { MangaSummary } from "../../services/manga";
import WorkCover from "../common/WorkCover.vue";

withDefaults(defineProps<{
  manga: MangaSummary[];
  showUnreadCount?: boolean;
}>(), {
  showUnreadCount: false,
});

const emit = defineEmits<{
  openManga: [manga: MangaSummary];
}>();
</script>

<template>
  <div class="result-grid manga-grid">
    <el-card
      v-for="item in manga"
      :key="item.id"
      class="book-card manga-card"
      shadow="hover"
      tabindex="0"
      @click="emit('openManga', item)"
      @keydown.enter="emit('openManga', item)"
    >
      <WorkCover class="book-cover" :cover-url="item.thumbnailUrl" :title="item.title" />
      <div class="book-meta manga-card-copy">
        <strong>{{ item.title }}</strong>
        <span v-if="item.author">{{ item.author }}</span>
        <em v-if="showUnreadCount && item.unreadCount">{{ item.unreadCount }} 话未读</em>
      </div>
    </el-card>
  </div>
</template>
