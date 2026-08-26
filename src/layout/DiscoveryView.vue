<script setup lang="ts" generic="T">
import RankingDiscovery from "../components/discovery/RankingDiscovery.vue";
import RecommendDiscovery from "../components/discovery/RecommendDiscovery.vue";
import BookCollection from "./BookCollection.vue";
import ContentTabs from "./ContentTabs.vue";
import type { BookSearchMode } from "../domain/search";
import type { BookGridItem } from "../types/book";
import type {
  DiscoveryTab,
  DiscoveryViewProps,
} from "../types/discovery";

const props = defineProps<DiscoveryViewProps<T>>();

const emit = defineEmits<{
  "update:modelValue": [value: DiscoveryTab];
  "update:query": [value: string];
  "update:searchMode": [value: BookSearchMode];
  "update:rankingDays": [value: number];
  search: [page: number];
  open: [item: BookGridItem<T>];
  retry: [region: DiscoveryTab];
}>();
</script>

<template>
  <section class="discovery-view">
    <ContentTabs
      :model-value="modelValue"
      :tabs="tabs"
      :search-label="search.searchLabel ?? '搜索作品'"
      :query="search.query"
      :search-mode="search.searchMode"
      :search-loading="search.loading"
      :two-primary="twoPrimary"
      :swipe="swipe"
      @search="emit('search', 1)"
      @update:model-value="emit('update:modelValue', $event)"
      @update:query="emit('update:query', $event)"
      @update:search-mode="emit('update:searchMode', $event)"
    />

    <RecommendDiscovery
      v-if="modelValue === 'recommend'"
      :blocks="recommend.blocks"
      :loading="recommend.loading"
      :error="recommend.error"
      :empty-message="recommend.emptyMessage ?? '暂无推荐内容'"
      :error-title="recommend.errorTitle ?? '推荐加载失败'"
      @open="emit('open', $event)"
      @retry="emit('retry', 'recommend')"
    />

    <RankingDiscovery
      v-else-if="modelValue === 'ranking'"
      :items="ranking.items"
      :loading="ranking.loading"
      :error="ranking.error"
      :empty-message="ranking.emptyMessage ?? '这个榜单还没有作品'"
      :error-title="ranking.errorTitle ?? '榜单加载失败'"
      :periods="ranking.periods"
      :days="ranking.days"
      @open="emit('open', $event)"
      @retry="emit('retry', 'ranking')"
      @update:days="emit('update:rankingDays', $event)"
    />

    <BookCollection
      v-else
      :items="search.result?.items ?? null"
      :pagination="search.result?.pagination"
      :loading="search.loading"
      :error="search.error"
      :prompt-message="search.promptMessage ?? '输入作品名、作者或标签开始搜索'"
      :empty-message="search.emptyMessage ?? '没有找到匹配的作品'"
      :error-title="search.errorTitle ?? '搜索失败'"
      grid-class="discovery-grid"
      @open="emit('open', $event)"
      @retry="emit('retry', 'search')"
      @load-more="emit('search', $event)"
    />
  </section>
</template>
