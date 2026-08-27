<script setup lang="ts">
import { ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { rankingPeriods, useDiscovery } from "../../composables/useDiscovery";
import type { NovelSummary } from "../../domain/novel";
import DiscoveryView from "../../layout/DiscoveryView.vue";
import type { BookGridItem } from "../../types/book";
import type { DiscoveryTab, DiscoveryTabOption } from "../../types/discovery";
import { novelDiscoveryAdapter } from "../../composables/discoveryAdapters";
import { useDiscoveryPresentation } from "../../composables/useDiscoveryPresentation";

const discovery = useDiscovery(novelDiscoveryAdapter);

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
const { recommend, ranking, search, retryDiscovery } = useDiscoveryPresentation(
  discovery,
  toBookItem,
  { ranking: { periods: rankingPeriods } },
);

function handleSearch(page: number): void {
  activeTab.value = "search";
  if (route.name !== "novel-search") {
    void router.push({ name: "novel-search" });
  }
  void discovery.runSearch(page);
}

function openNovel(novel: NovelSummary): void {
  void router.push({
    name: "detail",
    params: { bookId: novel.id },
    query: { from: route.name === "novel-search" ? "novel-search" : "novels" },
  });
}

function selectDiscoveryTab(tab: DiscoveryTab): void {
  activeTab.value = tab;
  if (tab !== "search" && route.name === "novel-search") {
    void router.replace({ name: "novels" });
  }
}

function selectRankingPeriod(days: number) {
  if (days === discovery.rankingDays.value) return;
  discovery.rankingDays.value = days;
  void discovery.loadRanking(days);
}

watch(activeTab, (tab) => {
  if (tab === "recommend" && discovery.recommendations.value.length === 0 && !discovery.loading.value.recommend && !discovery.errors.value.recommend) {
    void discovery.loadRecommendations();
  }
  if (tab === "ranking" && !discovery.ranking.value && !discovery.loading.value.ranking && !discovery.errors.value.ranking) {
    void discovery.loadRanking();
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
    :recommend="recommend"
    :ranking="ranking"
    :search="search"
    two-primary
    swipe
    @update:model-value="selectDiscoveryTab"
    @update:query="discovery.searchQuery.value = $event"
    @update:search-mode="discovery.searchMode.value = $event"
    @update:ranking-days="selectRankingPeriod"
    @search="handleSearch"
    @open="openNovel($event.data)"
    @retry="retryDiscovery"
  />
</template>
