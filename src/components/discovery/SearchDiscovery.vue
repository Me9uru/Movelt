<script setup lang="ts" generic="T">
import BookCollection from "../../layout/BookCollection.vue";
import type { BookSearchMode } from "../../domain/search";
import type { SearchDiscoveryModel } from "../../types/discovery";
import { useDiscoveryContext } from "../../composables/discovery/useDiscoveryContext";
import BookSearchBar from "./BookSearchBar.vue";

defineProps<SearchDiscoveryModel<T>>();
const discoveryContext = useDiscoveryContext();

const emit = defineEmits<{
  open: [item: T];
  retry: [];
  search: [page: number];
  "update:query": [value: string];
  "update:searchMode": [value: BookSearchMode];
}>();
</script>

<template>
  <BookSearchBar
    class="discovery-search discovery-search-page"
    :model-value="query"
    :search-mode="searchMode"
    :loading="loading"
    @update:model-value="emit('update:query', $event)"
    @update:search-mode="emit('update:searchMode', $event)"
    @clear="emit('update:query', '')"
    @submit="emit('search', 1)"
  />
  <BookCollection
    :items="result?.items ?? null"
    :pagination="result?.pagination"
    :loading="loading"
    :error="error"
    :prompt-message="`输入${discoveryContext.subjectLabel}名、作者或标签开始搜索`"
    :empty-message="`没有找到匹配的${discoveryContext.subjectLabel}`"
    :error-title="`${discoveryContext.subjectLabel}搜索失败`"
    grid-class="discovery-grid"
    @open="emit('open', $event.data)"
    @retry="emit('retry')"
    @load-more="emit('search', $event)"
  />
</template>
