<script setup lang="ts" generic="T">
import { computed } from "vue";
import BookCollection from "../book/BookCollection.vue";
import type { BookCollectionState, BookGridItem } from "../../types/book";

interface BookshelfCollectionProps<T> {
  items: BookGridItem<T>[];
  loading: boolean;
  filtering: boolean;
  emptyTitle: string;
}

const props = defineProps<BookshelfCollectionProps<T>>();

const emptyState = computed<BookCollectionState>(() => ({
  icon: props.filtering ? "magnify" : "bookmark",
  title: props.filtering ? "没有找到匹配的书籍" : props.emptyTitle,
  description: props.filtering
    ? "换个关键词试试看。"
    : "把想看的作品收藏起来，方便下次继续阅读。",
}));

const emit = defineEmits<{
  open: [item: T];
}>();
</script>

<template>
  <BookCollection
    class="bookshelf-collection"
    :items="items"
    :loading="loading"
    :disabled="loading"
    :empty-state="emptyState"
    @open="emit('open', $event.data)"
  />
</template>
