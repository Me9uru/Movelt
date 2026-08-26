import { ref, type Ref } from "vue";

import type { DiscoveryList, RecommendBlock } from "../domain/discovery";
import type { BookSearchMode } from "../domain/search";
import type { DiscoveryAdapter, DiscoveryRegion } from "../types/discovery";
import { getErrorMessage } from "../utils/error";

export const rankingPeriods = [
  { value: 7, label: "近 7 天" },
  { value: 30, label: "近 30 天" },
  { value: 365, label: "近一年" },
];

export interface DiscoveryState<T> {
  recommendations: Ref<RecommendBlock<T>[]>;
  ranking: Ref<T[] | null>;
  search: Ref<DiscoveryList<T> | null>;
  rankingDays: Ref<number>;
  searchQuery: Ref<string>;
  searchMode: Ref<BookSearchMode>;
  loading: Ref<Record<DiscoveryRegion, boolean>>;
  errors: Ref<Record<DiscoveryRegion, string>>;
  loadRecommendations: () => Promise<void>;
  loadRanking: (days?: number) => Promise<void>;
  runSearch: (page?: number) => Promise<void>;
}

export function useDiscovery<T>(
  adapter: DiscoveryAdapter<T>,
): DiscoveryState<T> {
  // ref 的 UnwrapRef 会递归展开泛型结构，导致 .value 类型与 T 不一致；断言回
  // 原始类型以便向 adapter 透传与增量追加。
  const recommendations = ref<RecommendBlock<T>[]>([]) as Ref<
    RecommendBlock<T>[]
  >;
  const ranking = ref<T[] | null>(null) as Ref<T[] | null>;
  const search = ref<DiscoveryList<T> | null>(
    null,
  ) as Ref<DiscoveryList<T> | null>;
  const rankingDays = ref(7);
  const searchQuery = ref("");
  const searchMode = ref<BookSearchMode>("title");
  const loading = ref<Record<DiscoveryRegion, boolean>>({
    recommend: false,
    ranking: false,
    search: false,
  });
  const errors = ref<Record<DiscoveryRegion, string>>({
    recommend: "",
    ranking: "",
    search: "",
  });
  async function run(region: DiscoveryRegion, task: () => Promise<void>) {
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
      // target 是 ref 持有的 reactive 代理本身；adapter 直接向它 push 即可触发
      // 渲染（漫画两阶段加载：最近更新先渲染、热门/新入库随后追加）。
      recommendations.value.splice(0, recommendations.value.length);
      await adapter.loadRecommendations(recommendations.value);
    });
  }
  async function loadRanking(days = rankingDays.value) {
    await run("ranking", async () => {
      ranking.value = await adapter.loadRanking(days);
    });
  }
  async function runSearch(page = 1) {
    if (!searchQuery.value.trim()) return;
    await run("search", async () => {
      const result = await adapter.search(
        searchQuery.value.trim(),
        page,
        searchMode.value,
      );
      if (page <= 1 || !search.value) {
        search.value = result;
      } else {
        // 无限滚动：同一搜索词翻页时在现有列表上追加，并推进分页游标。
        search.value.items.push(...result.items);
        search.value.pagination = result.pagination;
      }
    });
  }
  return {
    recommendations,
    ranking,
    search,
    rankingDays,
    searchQuery,
    searchMode,
    loading,
    errors,
    loadRecommendations,
    loadRanking,
    runSearch,
  };
}
