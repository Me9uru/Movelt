<script setup lang="ts">
import { watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import RankingDiscovery from "../../components/discovery/RankingDiscovery.vue";
import RecommendDiscovery from "../../components/discovery/RecommendDiscovery.vue";
import SearchDiscovery from "../../components/discovery/SearchDiscovery.vue";
import { comicDiscoveryAdapter } from "../../composables/discovery/adapters";
import { discoveryTabs } from "../../composables/discovery/config";
import { useDiscovery } from "../../composables/discovery/useDiscovery";
import { provideDiscoveryContext } from "../../composables/discovery/useDiscoveryContext";
import { useDiscoveryTabRoute } from "../../composables/discovery/useDiscoveryTabRoute";
import { useDiscoveryPresentation } from "../../composables/discovery/useDiscoveryPresentation";
import type { ComicSummary } from "../../domain/comic";
import DiscoveryLayout from "../../layout/DiscoveryLayout.vue";
import type { BookGridItem } from "../../types/book";
import { useDiscoveryScroll } from "../../composables/discovery/useDiscoveryScroll";
import { useDiscoverySearchStore } from "../../stores/discoverySearch";

const toBookItem = (comic: ComicSummary): BookGridItem<ComicSummary> => {
  return { id: comic.id, title: comic.title, coverUrl: comic.coverUrl, data: comic };
}

const route = useRoute();
const router = useRouter();
const discoverySearch = useDiscoverySearchStore();
const discovery = useDiscovery(comicDiscoveryAdapter, discoverySearch.comic);
provideDiscoveryContext("comic");
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
const { activeTab, selectTab } = useDiscoveryTabRoute("comic");
useDiscoveryScroll(discoverySearch.comic, () => activeTab.value === "search");
const { recommend, ranking, search, retryDiscovery } = useDiscoveryPresentation(
  discovery,
  toBookItem,
);

watch(activeTab, (tab) => {
  if (tab === "recommend" && discovery.recommendations.value.length === 0) {
    void discovery.loadRecommendations();
  }
  if (tab === "ranking" && !discovery.ranking.value) {
    void discovery.loadRanking();
  }
  if (tab === "search" && discovery.searchQuery.value.trim() && !discovery.search.value && !discovery.loading.value.search) {
    void discovery.runSearch();
  }
}, { immediate: true });

const handleSearch = async (page: number): Promise<void> => {
  const query = discovery.searchQuery.value.trim();
  if (!query) return;
  await router.replace({
    name: "comic",
    query: {
      ...route.query,
      tab: "search",
      q: query,
      mode: discovery.searchMode.value,
    },
  });
  await discovery.runSearch(page);
}
const openComic = (comic: ComicSummary): void => {
  void router.push({
    name: "comic-detail",
    params: { comicId: comic.id },
    query: activeTab.value === "search"
      ? {
          tab: "search",
          q: discovery.searchQuery.value.trim(),
          mode: discovery.searchMode.value,
        }
      : {},
  });
}

</script>

<template>
  <DiscoveryLayout
    :model-value="activeTab"
    :tabs="discoveryTabs"
    @update:model-value="selectTab"
  >
    <RecommendDiscovery
      v-if="activeTab === 'recommend'"
      v-bind="recommend"
      @open="openComic"
      @retry="retryDiscovery('recommend')"
    />
    <RankingDiscovery
      v-else-if="activeTab === 'ranking'"
      v-bind="ranking"
      @open="openComic"
      @retry="retryDiscovery('ranking')"
    />
    <SearchDiscovery
      v-else
      v-bind="search"
      @update:query="discovery.searchQuery.value = $event"
      @update:search-mode="discovery.searchMode.value = $event"
      @search="handleSearch"
      @open="openComic"
      @retry="retryDiscovery('search')"
    />
  </DiscoveryLayout>
</template>
