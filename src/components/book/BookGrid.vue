<script setup lang="ts" generic="T">
import BookCover from "./BookCover.vue";
import type { BookGridItem } from "../../types/book";

defineProps<{ items: BookGridItem<T>[]; loading?: boolean }>();
const emit = defineEmits<{ open: [item: BookGridItem<T>] }>();
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
      <BookCover class="book-cover" :cover-url="item.coverUrl" :title="item.title" />
      <div class="book-meta"><strong>{{ item.title }}</strong></div>
    </button>
  </div>
</template>
