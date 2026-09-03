import { ref, toRef, type Ref } from "vue";

import type {
  BookSearchMode,
  DiscoveryList,
  RecommendBlock,
} from "../../domain/discovery";
import type {
  DiscoveryAdapter,
  DiscoverySearchCache,
  DiscoveryTab,
} from "../../types/discovery";
import { getErrorMessage } from "../../utils/error";

export interface DiscoveryState<T> {
  recommendations: Ref<RecommendBlock<T>[]>;
  ranking: Ref<T[] | null>;
  search: Ref<DiscoveryList<T> | null>;
  rankingDays: Ref<number>;
  searchQuery: Ref<string>;
  searchMode: Ref<BookSearchMode>;
  loading: Ref<Record<DiscoveryTab, boolean>>;
  errors: Ref<Record<DiscoveryTab, string>>;
  loadRecommendations: () => Promise<void>;
  loadRanking: (days?: number) => Promise<void>;
  runSearch: (page?: number) => Promise<void>;
}

export const useDiscovery = <T>(
  adapter: DiscoveryAdapter<T>,
  searchCache?: DiscoverySearchCache<T>,
): DiscoveryState<T> => {
  // ref 的 UnwrapRef 会递归展开泛型结构，导致 .value 类型与 T 不一致；断言回
  // 原始类型以便向 adapter 透传与增量追加。
  const recommendations = ref<RecommendBlock<T>[]>([]) as Ref<
    RecommendBlock<T>[]
  >;
  const ranking = ref<T[] | null>(null) as Ref<T[] | null>;
  const search = searchCache
    ? toRef(searchCache, "search")
    : ref<DiscoveryList<T> | null>(null) as Ref<DiscoveryList<T> | null>;
  const rankingDays = ref(7);
  const searchQuery = searchCache
    ? toRef(searchCache, "searchQuery")
    : ref("");
  const searchMode = searchCache
    ? toRef(searchCache, "searchMode")
    : ref<BookSearchMode>("title");
  const loading = ref<Record<DiscoveryTab, boolean>>({
    recommend: false,
    ranking: false,
    search: false,
  });
  const errors = ref<Record<DiscoveryTab, string>>({
    recommend: "",
    ranking: "",
    search: "",
  });
  const run = async (region: DiscoveryTab, task: () => Promise<void>) => {
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
  const loadRecommendations = async () => {
    await run("recommend", async () => {
      const result: RecommendBlock<T>[] = [];
      await adapter.loadRecommendations(result);
      recommendations.value.splice(0, recommendations.value.length, ...result);
    });
  }
  const loadRanking = async (days = rankingDays.value) => {
    await run("ranking", async () => {
      const result = await adapter.loadRanking(days);
      ranking.value = result;
    });
  }
  const runSearch = async (page = 1) => {
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
