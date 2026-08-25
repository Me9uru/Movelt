<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { rankingPeriods } from "../../composables/useDiscovery";
import type { DiscoveryList, NovelSummary, RecommendBlock } from "../../domain/novel";
import type { BookSearchMode } from "../../domain/search";
import DiscoveryView from "../../components/book/DiscoveryView.vue";
import type { BookGridItem } from "../../types/book";
import type { DiscoveryTab, DiscoveryTabOption } from "../../types/discovery";

const props = defineProps<{
  unavailableMessage: string;
  recommendations: RecommendBlock<NovelSummary>[];
  ranking: NovelSummary[] | null;
  searchResult: DiscoveryList<NovelSummary> | null;
  rankingDays: number;
  searchQuery: string;
  searchMode: BookSearchMode;
  loading: Record<DiscoveryTab, boolean>;
  errors: Record<DiscoveryTab, string>;
}>();

const emit = defineEmits<{
  initialize: [];
  retryRecommendations: [];
  loadRanking: [days?: number];
  search: [page: number];
  openNovel: [novel: NovelSummary];
  "update:rankingDays": [value: number];
  "update:searchQuery": [value: string];
  "update:searchMode": [value: BookSearchMode];
}>();

function toBookItem(novel: NovelSummary): BookGridItem<NovelSummary> {
  return {
    id: `${novel.source}:${novel.id}`,
    title: novel.title,
    coverUrl: novel.cover_url,
    data: novel,
  };
}

const route = useRoute();
const router = useRouter();

const discoveryTabs: DiscoveryTabOption[] = [
  { name: "recommend", label: "推荐" },
  { name: "ranking", label: "排行榜" },
];
const activeTab = ref<DiscoveryTab>(
  route.name === "novel-search" ? "search" : "recommend",
);
const blocks = computed(() =>
  props.recommendations.map((block) => ({
    title: block.title,
    items: block.items.map(toBookItem),
  })),
);
const ranking = computed(() => props.ranking?.map(toBookItem) ?? null);
const searchResult = computed(() =>
  props.searchResult
    ? { items: props.searchResult.items.map(toBookItem), pagination: props.searchResult.pagination }
    : null,
);

function handleSearch(page: number): void {
  activeTab.value = "search";
  if (route.name !== "novel-search") {
    void router.push({ name: "novel-search" });
  }
  emit("search", page);
}

function selectDiscoveryTab(tab: DiscoveryTab): void {
  activeTab.value = tab;
  if (tab !== "search" && route.name === "novel-search") {
    void router.replace({ name: "novels" });
  }
}

function selectRankingPeriod(days: number) {
  if (days === props.rankingDays) return;

  emit("update:rankingDays", days);
  emit("loadRanking", days);
}

watch(activeTab, (tab) => {
  if (tab === "recommend" && props.recommendations.length === 0 && !props.loading.recommend && !props.errors.recommend) {
    emit("retryRecommendations");
  }
  if (tab === "ranking" && !props.ranking && !props.loading.ranking && !props.errors.ranking) {
    emit("loadRanking");
  }
}, { immediate: true });

watch(
  () => route.name,
  (name) => {
    if (name === "novel-search") activeTab.value = "search";
  },
);
</script>

<template>
  <DiscoveryView
    :model-value="activeTab"
    :tabs="discoveryTabs"
    :blocks="blocks"
    :ranking="ranking"
    :search-result="searchResult"
    :loading="loading"
    :errors="errors"
    :query="searchQuery"
    :search-mode="searchMode"
    :ranking-periods="rankingPeriods"
    :ranking-days="rankingDays"
    :unavailable-message="unavailableMessage"
    two-primary
    swipe
    @initialize="emit('initialize')"
    @update:model-value="selectDiscoveryTab"
    @update:query="emit('update:searchQuery', $event)"
    @update:search-mode="emit('update:searchMode', $event)"
    @update:ranking-days="selectRankingPeriod"
    @search="handleSearch"
    @open="emit('openNovel', $event.data)"
    @retry-recommend="emit('retryRecommendations')"
    @retry-ranking="emit('loadRanking')"
    @retry-search="emit('search', searchResult?.pagination?.page || 1)"
  />
</template>
