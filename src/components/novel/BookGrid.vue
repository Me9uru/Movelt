<script setup lang="ts">
import { computed } from "vue";
import type { NovelSummary } from "../../domain/content";
import WorkGrid, { type WorkGridItem } from "../common/WorkGrid.vue";

const props = defineProps<{ books: NovelSummary[]; loading?: boolean }>();
const emit = defineEmits<{ openNovel: [novel: NovelSummary] }>();

const items = computed<(WorkGridItem & { novel: NovelSummary })[]>(() => props.books.map((novel) => ({
  id: `${novel.source}:${novel.id}`,
  title: novel.title,
  coverUrl: novel.cover_url,
  novel,
})));

function open(item: WorkGridItem): void {
  const match = items.value.find((entry) => entry.id === item.id);
  if (match) emit("openNovel", match.novel);
}
</script>

<template>
  <WorkGrid :items="items" :loading="loading" @open="open" />
</template>
