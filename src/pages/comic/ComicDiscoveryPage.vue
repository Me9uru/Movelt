<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { comicDiscoveryAdapter } from "../../composables/discoveryAdapters";
import { useDiscovery } from "../../composables/useDiscovery";
import { useDiscoveryPresentation } from "../../composables/useDiscoveryPresentation";
import type { ComicSummary } from "../../domain/comic";
import DiscoveryView from "../../layout/DiscoveryView.vue";
import type { BookGridItem } from "../../types/book";
import type {
  DiscoveryTab,
  DiscoveryTabOption,
} from "../../types/discovery";

function toBookItem(comic: ComicSummary): BookGridItem<ComicSummary> {
  return { id: comic.id, title: comic.title, coverUrl: comic.coverUrl, data: comic };
}

const router = useRouter();
const discovery = useDiscovery(comicDiscoveryAdapter);
const activeTab = ref<DiscoveryTab>("recommend");
const comicTabs: DiscoveryTabOption[] = [
  { name: "recommend", label: "精选" },
  { name: "ranking", label: "排行榜" },
];
const { recommend, ranking, search, retryDiscovery } = useDiscoveryPresentation(
  discovery,
  toBookItem,
  {
    recommend: { errorTitle: "漫画加载失败" },
    ranking: {
      emptyMessage: "没有找到匹配的漫画",
      errorTitle: "漫画加载失败",
    },
    search: {
      searchLabel: "搜索漫画",
      emptyMessage: "没有找到匹配的漫画",
      errorTitle: "漫画加载失败",
    },
  },
);

watch(activeTab, (tab) => {
  if (tab === "recommend" && discovery.recommendations.value.length === 0) void discovery.loadRecommendations();
  if (tab === "ranking") void discovery.loadRanking();
});
onMounted(() => void discovery.loadRecommendations());

function handleSearch(page: number): void {
  activeTab.value = "search";
  void discovery.runSearch(page);
}
function openComic(comic: ComicSummary): void {
  void router.push({ name: "comic-detail", params: { comicId: comic.id } });
}

</script>

<template>
  <DiscoveryView
    v-model="activeTab"
    :tabs="comicTabs"
    :recommend="recommend"
    :ranking="ranking"
    :search="search"
    two-primary
    @search="handleSearch"
    @update:query="discovery.searchQuery.value = $event"
    @update:search-mode="discovery.searchMode.value = $event"
    @open="openComic($event.data)"
    @retry="retryDiscovery"
  />
</template>
