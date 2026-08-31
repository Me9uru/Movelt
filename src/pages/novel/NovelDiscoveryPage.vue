<script setup lang="ts">
import { watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import RankingDiscovery from "../../components/discovery/RankingDiscovery.vue";
import RecommendDiscovery from "../../components/discovery/RecommendDiscovery.vue";
import SearchDiscovery from "../../components/discovery/SearchDiscovery.vue";
import { discoveryTabs, rankingPeriods } from "../../composables/discovery/config";
import { useDiscovery } from "../../composables/discovery/useDiscovery";
import { provideDiscoveryContext } from "../../composables/discovery/useDiscoveryContext";
import { useDiscoveryTabRoute } from "../../composables/discovery/useDiscoveryTabRoute";
import type { NovelSummary } from "../../domain/novel";
import DiscoveryLayout from "../../layout/DiscoveryLayout.vue";
import type { BookGridItem } from "../../types/book";
import { novelDiscoveryAdapter } from "../../composables/discovery/adapters";
import { useDiscoveryPresentation } from "../../composables/discovery/useDiscoveryPresentation";
import { useDiscoveryScroll } from "../../composables/discovery/useDiscoveryScroll";
import { useDiscoverySearchStore } from "../../stores/discoverySearch";

const route = useRoute();
const router = useRouter();
const discoverySearch = useDiscoverySearchStore();
const discovery = useDiscovery(novelDiscoveryAdapter, discoverySearch.novel);
provideDiscoveryContext("novel");

const routeSearchMode = route.query.mode;
if (!discovery.searchQuery.value && typeof route.query.q === "string") {
  discovery.searchQuery.value = route.query.q;
}
if (
  routeSearchMode === "title" ||
  routeSearchMode === "author" ||
  routeSearchMode === "tags"
) {
  discovery.searchMode.value = routeSearchMode;
}

const toBookItem = (novel: NovelSummary): BookGridItem<NovelSummary> => {
  return {
    id: `${novel.source}:${novel.id}`,
    title: novel.title,
    coverUrl: novel.cover_url,
    data: novel,
  };
}

const { activeTab, selectTab } = useDiscoveryTabRoute("novels");
useDiscoveryScroll(discoverySearch.novel, () => activeTab.value === "search");
const { recommend, ranking, search, retryDiscovery } = useDiscoveryPresentation(
  discovery,
  toBookItem,
  { rankingPeriods },
);

const handleSearch = async (page: number): Promise<void> => {
  const query = discovery.searchQuery.value.trim();
  if (!query) return;
  await router.replace({
    name: "novels",
    query: {
      ...route.query,
      tab: "search",
      q: query,
      mode: discovery.searchMode.value,
    },
  });
  await discovery.runSearch(page);
}

const openNovel = (novel: NovelSummary): void => {
  void router.push({
    name: "detail",
    params: { bookId: novel.id },
    query: {
      from: "novels",
      ...(activeTab.value === "search"
        ? {
            tab: "search",
            q: discovery.searchQuery.value.trim(),
            mode: discovery.searchMode.value,
          }
        : {}),
    },
  });
}

const selectRankingPeriod = (days: number) => {
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
  if (tab === "search" && discovery.searchQuery.value.trim() && !discovery.search.value && !discovery.loading.value.search) {
    void discovery.runSearch();
  }
}, { immediate: true });

</script>

<template>
  <DiscoveryLayout
    :model-value="activeTab"
    :tabs="discoveryTabs"
    swipe
    @update:model-value="selectTab"
  >
    <RecommendDiscovery
      v-if="activeTab === 'recommend'"
      v-bind="recommend"
      @open="openNovel"
      @retry="retryDiscovery('recommend')"
    />
    <RankingDiscovery
      v-else-if="activeTab === 'ranking'"
      v-bind="ranking"
      @open="openNovel"
      @retry="retryDiscovery('ranking')"
      @update:days="selectRankingPeriod"
    />
    <SearchDiscovery
      v-else
      v-bind="search"
      @update:query="discovery.searchQuery.value = $event"
      @update:search-mode="discovery.searchMode.value = $event"
      @search="handleSearch"
      @open="openNovel"
      @retry="retryDiscovery('search')"
    />
  </DiscoveryLayout>
</template>
