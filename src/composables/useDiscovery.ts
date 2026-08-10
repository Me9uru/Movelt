import { ref } from "vue";
import {
  getCategory,
  getDiscoveryHealth,
  getRanking,
  getRecommendations,
  searchDiscovery,
  type DiscoveryList,
  type HealthStatus,
  type RankingSort,
  type RecommendBlock,
} from "../services/novel";

export const categoryPresets = [
  "奇幻", "校园", "爱情", "冒险", "穿越", "魔法",
  "战斗", "科幻", "悬疑", "治愈", "搞笑", "后宫",
];

export const rankingSorts: { value: RankingSort; label: string }[] = [
  { value: "allvisit", label: "总排行榜" }, { value: "allvote", label: "总推荐榜" },
  { value: "monthvisit", label: "月排行榜" }, { value: "monthvote", label: "月推荐榜" },
  { value: "weekvisit", label: "周排行榜" }, { value: "weekvote", label: "周推荐榜" },
  { value: "dayvisit", label: "日排行榜" }, { value: "dayvote", label: "日推荐榜" },
  { value: "postdate", label: "最新入库" }, { value: "lastupdate", label: "最近更新" },
  { value: "goodnum", label: "总收藏榜" }, { value: "size", label: "字数排行" },
  { value: "fullflag", label: "完结作品" }, { value: "anime", label: "已动画化" },
];

type Region = "recommend" | "ranking" | "category" | "search";

export function useDiscovery() {
  const health = ref<HealthStatus | null>(null);
  const unavailableMessage = ref("");
  const recommendations = ref<RecommendBlock[]>([]);
  const ranking = ref<DiscoveryList | null>(null);
  const category = ref<DiscoveryList | null>(null);
  const search = ref<DiscoveryList | null>(null);
  const rankingSort = ref<RankingSort>("allvisit");
  const categorySort = ref<RankingSort>("lastupdate");
  const categoryTag = ref("奇幻");
  const customTag = ref("");
  const searchQuery = ref("");
  const loading = ref<Record<Region, boolean>>({ recommend: false, ranking: false, category: false, search: false });
  const errors = ref<Record<Region, string>>({ recommend: "", ranking: "", category: "", search: "" });
  const sequences: Record<Region, number> = { recommend: 0, ranking: 0, category: 0, search: 0 };

  function describe(error: unknown): string {
    if (typeof error === "string") return error;
    if (error && typeof error === "object") {
      const value = error as { code?: unknown; message?: unknown };
      if (value.code === "NOT_LOGGED_IN") return "Wenku8 API 登录态不可用，请修复账号配置并重启 API。";
      if (typeof value.message === "string") return value.message;
      if (typeof value.code === "string") return value.code;
    }
    return "请求失败，请稍后重试。";
  }

  async function run(region: Region, task: () => Promise<void>): Promise<void> {
    const sequence = ++sequences[region];
    loading.value[region] = true;
    errors.value[region] = "";
    try {
      await task();
    } catch (error) {
      if (sequence !== sequences[region]) return;
      const message = describe(error);
      errors.value[region] = message;
      if (message.includes("登录态")) unavailableMessage.value = message;
    } finally {
      if (sequence === sequences[region]) loading.value[region] = false;
    }
  }

  async function loadRecommendations(): Promise<void> {
    await run("recommend", async () => { recommendations.value = await getRecommendations(); });
  }
  async function loadRanking(page = 1): Promise<void> {
    await run("ranking", async () => { ranking.value = await getRanking(rankingSort.value, page); });
  }
  async function loadCategory(page = 1): Promise<void> {
    const tag = customTag.value.trim() || categoryTag.value;
    await run("category", async () => { category.value = await getCategory(tag, categorySort.value, page); });
  }
  async function runSearch(page = 1): Promise<void> {
    const query = searchQuery.value.trim();
    if (!query) return;
    await run("search", async () => { search.value = await searchDiscovery(query, page); });
  }
  async function initialize(): Promise<void> {
    try {
      health.value = await getDiscoveryHealth();
      if (!health.value.logged_in) {
        unavailableMessage.value = `Wenku8 API（${health.value.base_url}）未登录，请修复登录配置并重启 API。`;
        return;
      }
      unavailableMessage.value = "";
      await loadRecommendations();
    } catch (error) {
      unavailableMessage.value = `无法连接 Wenku8 API：${describe(error)}`;
    }
  }

  function selectCategory(tag: string): void {
    categoryTag.value = tag;
    customTag.value = "";
    void loadCategory(1);
  }

  return {
    health, unavailableMessage, recommendations, ranking, category, search,
    rankingSort, categorySort, categoryTag, customTag, searchQuery,
    loading, errors, initialize, loadRecommendations, loadRanking, loadCategory,
    runSearch, selectCategory,
  };
}
