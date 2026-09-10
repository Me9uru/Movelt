<script setup lang="ts">
import { useRouter } from "vue-router";
import RankingDiscovery from "../../components/discovery/RankingDiscovery.vue";
import RecommendDiscovery from "../../components/discovery/RecommendDiscovery.vue";
import SearchDiscovery from "../../components/discovery/SearchDiscovery.vue";
import { discoveryTabs, rankingPeriods } from "../../composables/discovery/config";
import { useDiscoveryPage } from "../../composables/discovery/useDiscoveryPage";
import { novelDiscoveryAdapter } from "../../composables/discovery/adapters";
import { useDiscoveryPresentation } from "../../composables/discovery/useDiscoveryPresentation";
import { useDiscoverySearchStore } from "../../stores/discoverySearch";
import { toNovelGridItem } from "../../utils/bookPresentation";
import type { NovelSummary } from "../../domain/novel";
import TabbedPageLayout from "../../layout/TabbedPageLayout.vue";

const router = useRouter();
const cache = useDiscoverySearchStore();
const { discovery, activeTab, selectTab, handleSearch, detailQuery } =
  useDiscoveryPage("novel", novelDiscoveryAdapter, cache.novel);
const { recommend, ranking, search, retryDiscovery } = useDiscoveryPresentation(
  discovery, toNovelGridItem, { rankingPeriods },
);

const openNovel = (item: NovelSummary): void => {
  void router.push({
    name: "novel-detail",
    params: { bookId: item.id },
    query: detailQuery(),
  });
};

const selectRankingPeriod = (days: number): void => {
  if (days === discovery.rankingDays.value) return;
  discovery.rankingDays.value = days;
  void discovery.loadRanking(days);
};

</script>
<template>
  <TabbedPageLayout
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
  </TabbedPageLayout>
</template>
