<script setup lang="ts">
import WorkCover from "./WorkCover.vue";

export interface WorkGridItem {
  id: string;
  title: string;
  coverUrl: string | null;
}

defineProps<{ items: WorkGridItem[]; loading?: boolean }>();
const emit = defineEmits<{ open: [item: WorkGridItem] }>();
</script>

<template>
  <div v-if="loading" class="work-grid result-grid" aria-label="正在加载">
    <div v-for="item in 6" :key="item" class="book-card skeleton-card">
      <var-skeleton>
        <template #default>
          <div class="skeleton-cover" />
          <div style="width: 82%" />
        </template>
      </var-skeleton>
    </div>
  </div>
  <div v-else class="work-grid result-grid">
    <button
      v-for="item in items"
      :key="item.id"
      type="button"
      class="book-card"
      @click="emit('open', item)"
    >
      <WorkCover class="book-cover" :cover-url="item.coverUrl" :title="item.title" />
      <div class="book-meta"><strong>{{ item.title }}</strong></div>
    </button>
  </div>
</template>
