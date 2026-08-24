import { ref } from "vue";

import {
  getLatest,
  getRank,
  getRanking,
  searchDiscovery,
} from "../services/novel";
import type { DiscoveryList, NovelSummary, RecommendBlock } from "../domain/content";
import type { BookSearchMode } from "../domain/search";
import { getErrorMessage } from "../utils/error";

export const rankingPeriods = [
  { value: 7, label: "近 7 天" },
  { value: 30, label: "近 30 天" },
  { value: 365, label: "近一年" },
];
type Region = "recommend" | "ranking" | "search";

export function useDiscovery() {
  const unavailableMessage = ref("");
  const recommendations = ref<RecommendBlock[]>([]);
  const ranking = ref<NovelSummary[] | null>(null);
  const search = ref<DiscoveryList | null>(null);
  const rankingDays = ref(7);
  const searchQuery = ref("");
  const searchMode = ref<BookSearchMode>("title");
  const loading = ref<Record<Region, boolean>>({
    recommend: false,
    ranking: false,
    search: false,
  });
  const errors = ref<Record<Region, string>>({
    recommend: "",
    ranking: "",
    search: "",
  });
  async function run(region: Region, task: () => Promise<void>) {
    loading.value[region] = true;
    errors.value[region] = "";
    try {
      await task();
    } catch (error) {
      errors.value[region] = getErrorMessage(error);
    } finally {
      loading.value[region] = false;
    }
  }
  async function loadRecommendations() {
    await run("recommend", async () => {
      const [latest, popular, newest] = await Promise.all([
        getLatest(),
        getRanking("view"),
        getRanking("new"),
      ]);
      recommendations.value = [
        { title: "最近更新", items: latest.items.slice(0, 6) },
        { title: "热门作品", items: popular.items.slice(0, 6) },
        { title: "新入库", items: newest.items.slice(0, 6) },
      ];
    });
  }
  async function loadRanking(days = rankingDays.value) {
    await run("ranking", async () => {
      ranking.value = await getRank(days);
    });
  }
  async function runSearch(page = 1) {
    if (searchQuery.value.trim())
      await run("search", async () => {
        search.value = await searchDiscovery(searchQuery.value.trim(), page, searchMode.value);
      });
  }
  async function initialize() {
    unavailableMessage.value = "";
    await loadRecommendations();
  }
  return {
    unavailableMessage,
    recommendations,
    ranking,
    search,
    rankingDays,
    searchQuery,
    searchMode,
    loading,
    errors,
    initialize,
    loadRecommendations,
    loadRanking,
    runSearch,
  };
}
