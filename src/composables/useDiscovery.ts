import { ref } from "vue";

import { getLatest, getRank, getRanking, searchByTags, searchDiscovery, type DiscoveryList, type NovelSummary, type RecommendBlock } from "../services/novel";
import { getErrorMessage } from "../utils/error";

export const categoryPresets = ["校园", "恋爱", "奇幻", "异世界", "科幻", "悬疑"];
export const rankingPeriods = [
  { value: 7, label: "近 7 天" }, { value: 30, label: "近 30 天" }, { value: 365, label: "近一年" },
];
type Region = "recommend" | "ranking" | "category" | "search";

export function useDiscovery() {
  const unavailableMessage = ref("");
  const recommendations = ref<RecommendBlock[]>([]);
  const ranking = ref<NovelSummary[] | null>(null);
  const category = ref<DiscoveryList | null>(null);
  const search = ref<DiscoveryList | null>(null);
  const rankingDays = ref(7);
  const categoryTag = ref("");
  const customTag = ref("");
  const searchQuery = ref("");
  const loading = ref<Record<Region, boolean>>({ recommend: false, ranking: false, category: false, search: false });
  const errors = ref<Record<Region, string>>({ recommend: "", ranking: "", category: "", search: "" });
  async function run(region: Region, task: () => Promise<void>) {
    loading.value[region] = true; errors.value[region] = "";
    try { await task(); } catch (error) { errors.value[region] = getErrorMessage(error); } finally { loading.value[region] = false; }
  }
  async function loadRecommendations() {
    await run("recommend", async () => {
      const [latest, popular, newest] = await Promise.all([getLatest(), getRanking("view"), getRanking("new")]);
      recommendations.value = [
        { title: "最近更新", items: latest.items.slice(0, 6) },
        { title: "热门作品", items: popular.items.slice(0, 6) },
        { title: "新入库", items: newest.items.slice(0, 6) },
      ];
    });
  }
  async function loadRanking(days = rankingDays.value) { await run("ranking", async () => { ranking.value = await getRank(days); }); }
  async function loadCategory(page = 1) {
    const tags = customTag.value.trim();
    if (!tags) { category.value = null; return; }
    await run("category", async () => { category.value = await searchByTags(tags, page); });
  }
  async function runSearch(page = 1) { if (searchQuery.value.trim()) await run("search", async () => { search.value = await searchDiscovery(searchQuery.value.trim(), page); }); }
  async function initialize() { unavailableMessage.value = ""; await loadRecommendations(); }
  function selectCategory(tag: string) { categoryTag.value = tag; customTag.value = tag; void loadCategory(); }
  return { unavailableMessage, recommendations, ranking, category, search, rankingDays, categoryTag, customTag, searchQuery, loading, errors, initialize, loadRecommendations, loadRanking, loadCategory, runSearch, selectCategory };
}
