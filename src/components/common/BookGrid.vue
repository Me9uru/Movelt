<script setup lang="ts" generic="T">
import BookCover from "./BookCover.vue";
import type { BookGridItem, BookGridProps } from "../../types/book";

defineProps<BookGridProps<T>>();
const emit = defineEmits<{ open: [item: BookGridItem<T>] }>();
</script>

<template>
  <div class="book-grid result-grid">
    <var-button
      v-for="item in items"
      :key="item.id"
      text
      class="book-card"
      :class="{ 'book-card--disabled': disabled }"
      :disabled="disabled"
      @click="emit('open', item)"
    >
      <BookCover
        class="book-cover"
        :cover-url="item.coverUrl"
        :title="item.title"
      />
      <div class="book-meta">
        <strong>{{ item.title }}</strong>
        <span v-if="item.meta">{{ item.meta }}</span>
      </div>
    </var-button>
  </div>
</template>
