<script setup lang="ts">
import { useRouter } from "vue-router";
import RankingDiscovery from "../../components/discovery/RankingDiscovery.vue";
import RecommendDiscovery from "../../components/discovery/RecommendDiscovery.vue";
import SearchDiscovery from "../../components/discovery/SearchDiscovery.vue";
import { discoveryTabs } from "../../composables/discovery/config";
import { useDiscoveryPage } from "../../composables/discovery/useDiscoveryPage";
import { comicDiscoveryAdapter } from "../../composables/discovery/adapters";
import { useDiscoveryPresentation } from "../../composables/discovery/useDiscoveryPresentation";
import { useDiscoverySearchStore } from "../../stores/discoverySearch";
import { toComicGridItem } from "../../utils/bookPresentation";
import type { ComicSummary } from "../../domain/comic";
import TabbedPageLayout from "../../layout/TabbedPageLayout.vue";

const router = useRouter();
const cache = useDiscoverySearchStore();
const { discovery, activeTab, selectTab, handleSearch, detailQuery } =
  useDiscoveryPage("comic", comicDiscoveryAdapter, cache.comic);
const { recommend, ranking, search, retryDiscovery } = useDiscoveryPresentation(
  discovery, toComicGridItem,
);

const openComic = (item: ComicSummary): void => {
  void router.push({
    name: "comic-detail",
    params: { comicId: item.id },
    query: detailQuery(),
  });
};

</script>
<template>
  <TabbedPageLayout
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
  </TabbedPageLayout>
</template>
