<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { mangaDiscoveryAdapter } from "../../composables/discoveryAdapters";
import { useDiscovery } from "../../composables/useDiscovery";
import type { MangaSummary } from "../../domain/manga";
import DiscoveryView from "../../components/book/DiscoveryView.vue";
import type { BookGridItem } from "../../types/book";
import type {
  DiscoveryTab,
  DiscoveryTabOption,
  DiscoveryViewLabels,
} from "../../types/discovery";

function toBookItem(manga: MangaSummary): BookGridItem<MangaSummary> {
  return { id: manga.id, title: manga.title, coverUrl: manga.thumbnailUrl, data: manga };
}

const router = useRouter();
const discovery = useDiscovery(mangaDiscoveryAdapter);
const activeTab = ref<DiscoveryTab>("recommend");
const mangaTabs: DiscoveryTabOption[] = [
  { name: "recommend", label: "精选" },
  { name: "ranking", label: "排行榜" },
];
const labels: DiscoveryViewLabels = {
  searchLabel: "搜索漫画",
  rankingEmpty: "没有找到匹配的漫画",
  searchEmpty: "没有找到匹配的漫画",
  errorTitles: { recommend: "漫画加载失败", ranking: "漫画加载失败", search: "漫画加载失败" },
};
const blocks = computed(() =>
  discovery.recommendations.value.map((block) => ({
    title: block.title,
    items: block.items.map(toBookItem),
  })),
);
const ranking = computed(() => discovery.ranking.value?.map(toBookItem) ?? null);
const searchResult = computed(() =>
  discovery.search.value
    ? { items: discovery.search.value.items.map(toBookItem), pagination: discovery.search.value.pagination }
    : null,
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
function openManga(manga: MangaSummary): void {
  void router.push({ name: "manga-detail", params: { mangaId: manga.id } });
}
</script>

<template>
  <DiscoveryView
    v-model="activeTab"
    :tabs="mangaTabs"
    :blocks="blocks"
    :ranking="ranking"
    :search-result="searchResult"
    :loading="discovery.loading.value"
    :errors="discovery.errors.value"
    :query="discovery.searchQuery.value"
    :search-mode="discovery.searchMode.value"
    :labels="labels"
    two-primary
    @search="handleSearch"
    @update:query="discovery.searchQuery.value = $event"
    @update:search-mode="discovery.searchMode.value = $event"
    @open="openManga($event.data)"
  />
</template>
