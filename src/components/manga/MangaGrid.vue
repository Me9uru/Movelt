<script setup lang="ts">
import { computed } from "vue";
import type { MangaSummary } from "../../domain/content";
import WorkGrid, { type WorkGridItem } from "../common/WorkGrid.vue";

const props = defineProps<{ manga: MangaSummary[] }>();

const emit = defineEmits<{
  openManga: [manga: MangaSummary];
}>();

const items = computed<(WorkGridItem & { manga: MangaSummary })[]>(() => props.manga.map((manga) => ({
  id: manga.id,
  title: manga.title,
  coverUrl: manga.thumbnailUrl,
  manga,
})));

function open(item: WorkGridItem): void {
  const match = items.value.find((entry) => entry.id === item.id);
  if (match) emit("openManga", match.manga);
}
</script>

<template>
  <WorkGrid class="manga-grid" :items="items" @open="open" />
</template>
