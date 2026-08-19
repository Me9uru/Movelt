import { ref } from "vue";

import { getLatest, getRanking, searchDiscovery, type DiscoveryList, type RankingSort, type RecommendBlock } from "../services/novel";

export const categoryPresets: string[] = [];
export const rankingSorts: { value: RankingSort; label: string }[] = [
  { value: "latest", label: "最近更新" }, { value: "view", label: "热门作品" }, { value: "new", label: "最新入库" },
];
type Region = "recommend" | "ranking" | "category" | "search";

export function useDiscovery() {
  const unavailableMessage = ref("");
  const recommendations = ref<RecommendBlock[]>([]);
  const ranking = ref<DiscoveryList | null>(null);
  const category = ref<DiscoveryList | null>(null);
  const search = ref<DiscoveryList | null>(null);
  const rankingSort = ref<RankingSort>("latest");
  const categoryTag = ref("");
  const customTag = ref("");
  const searchQuery = ref("");
  const loading = ref<Record<Region, boolean>>({ recommend: false, ranking: false, category: false, search: false });
  const errors = ref<Record<Region, string>>({ recommend: "", ranking: "", category: "", search: "" });
  async function run(region: Region, task: () => Promise<void>) {
    loading.value[region] = true; errors.value[region] = "";
    try { await task(); } catch (error) { errors.value[region] = error instanceof Error ? error.message : "请求失败"; } finally { loading.value[region] = false; }
  }
  async function loadRecommendations() { await run("recommend", async () => { recommendations.value = [{ title: "最近更新", items: (await getLatest()).items }]; }); }
  async function loadRanking(page = 1) { await run("ranking", async () => { ranking.value = await getRanking(rankingSort.value, page); }); }
  async function loadCategory(page = 1) { await run("category", async () => { category.value = customTag.value.trim() ? await searchDiscovery(customTag.value.trim(), page) : await getLatest(page); }); }
  async function runSearch(page = 1) { if (searchQuery.value.trim()) await run("search", async () => { search.value = await searchDiscovery(searchQuery.value.trim(), page); }); }
  async function initialize() { unavailableMessage.value = ""; await loadRecommendations(); }
  function selectCategory(tag: string) { categoryTag.value = tag; customTag.value = tag; void loadCategory(); }
  return { unavailableMessage, recommendations, ranking, category, search, rankingSort, categoryTag, customTag, searchQuery, loading, errors, initialize, loadRecommendations, loadRanking, loadCategory, runSearch, selectCategory };
}
