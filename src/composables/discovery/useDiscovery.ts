import { onScopeDispose, ref, shallowRef, toRef, type Ref } from "vue";
import type { BookSearchMode, DiscoveryList, RecommendBlock } from "../../domain/discovery";
import type { DiscoveryAdapter, DiscoverySearchCache, DiscoveryTab } from "../../types/discovery";
import { getErrorMessage } from "../../utils/error";
import { createLatestRequest } from "../../utils/latestRequest";

export interface DiscoveryState<T> {
  recommendations: Ref<RecommendBlock<T>[]>;
  ranking: Ref<T[] | null>;
  search: Ref<DiscoveryList<T> | null>;
  submitted: Ref<DiscoverySearchCache<T>["submitted"]>;
  rankingDays: Ref<number>;
  searchQuery: Ref<string>;
  searchMode: Ref<BookSearchMode>;
  loading: Ref<Record<DiscoveryTab, boolean>>;
  errors: Ref<Record<DiscoveryTab, string>>;
  loadRecommendations: () => Promise<void>;
  loadRanking: (days?: number) => Promise<void>;
  runSearch: (page?: number) => Promise<void>;
  retry: (region: DiscoveryTab) => Promise<void>;
}

export const useDiscovery = <T>(
  adapter: DiscoveryAdapter<T>,
  searchCache?: DiscoverySearchCache<T>,
): DiscoveryState<T> => {
  const recommendations = shallowRef<RecommendBlock<T>[]>([]);
  const ranking = shallowRef<T[] | null>(null);
  const search = searchCache ? toRef(searchCache, "search") : shallowRef<DiscoveryList<T> | null>(null);
  const submitted = searchCache ? toRef(searchCache, "submitted") : ref<DiscoverySearchCache<T>["submitted"]>(null);
  const rankingDays = ref(7);
  const searchQuery = searchCache ? toRef(searchCache, "searchQuery") : ref("");
  const searchMode = searchCache ? toRef(searchCache, "searchMode") : ref<BookSearchMode>("title");
  const loading = ref<Record<DiscoveryTab, boolean>>({ recommend: false, ranking: false, search: false });
  const errors = ref<Record<DiscoveryTab, string>>({ recommend: "", ranking: "", search: "" });
  const requests = {
    recommend: createLatestRequest(),
    ranking: createLatestRequest(),
    search: createLatestRequest(),
  };
  const retries: Partial<Record<DiscoveryTab, () => Promise<void>>> = {};
  onScopeDispose(() => Object.values(requests).forEach((request) => request.invalidate()));

  const run = async (
    region: DiscoveryTab,
    task: (isCurrent: () => boolean) => Promise<void>,
    retry: () => Promise<void>,
  ): Promise<void> => {
    const isCurrent = requests[region].start();
    loading.value[region] = true;
    errors.value[region] = "";
    delete retries[region];
    try {
      await task(isCurrent);
    } catch (error) {
      if (!isCurrent()) return;
      errors.value[region] = getErrorMessage(error);
      retries[region] = retry;
    } finally {
      if (isCurrent()) loading.value[region] = false;
    }
  };

  const loadRecommendations = async (): Promise<void> => {
    if (loading.value.recommend) return;
    await run("recommend", async (isCurrent) => {
      const result = await adapter.loadRecommendations();
      if (isCurrent()) recommendations.value = result;
    }, loadRecommendations);
  };

  const loadRanking = async (days = rankingDays.value): Promise<void> => {
    await run("ranking", async (isCurrent) => {
      const result = await adapter.loadRanking(days);
      if (isCurrent()) ranking.value = result;
    }, () => loadRanking(days));
  };

  const searchPage = async (
    criteria: NonNullable<DiscoverySearchCache<T>["submitted"]>,
    page: number,
  ): Promise<void> => {
    await run("search", async (isCurrent) => {
      const result = await adapter.search(criteria.query, page, criteria.mode);
      // Clearing the session cache also invalidates a request's criteria.
      if (!isCurrent() || submitted.value?.query !== criteria.query || submitted.value.mode !== criteria.mode) return;
      search.value = page === 1
        ? result
        : { items: [...(search.value?.items ?? []), ...result.items], pagination: result.pagination };
    }, () => searchPage(criteria, page));
  };

  const runSearch = async (page = 1): Promise<void> => {
    if (page > 1) {
      if (loading.value.search || !submitted.value || !search.value ||
        page !== search.value.pagination.page + 1 || page > search.value.pagination.last) return;
      await searchPage({ ...submitted.value }, page);
      return;
    }
    const query = searchQuery.value.trim();
    if (!query) return;
    const criteria = { query, mode: searchMode.value };
    submitted.value = criteria;
    search.value = null;
    if (searchCache) searchCache.scrollTop = 0;
    await searchPage(criteria, 1);
  };

  const retry = async (region: DiscoveryTab): Promise<void> => {
    if (!loading.value[region]) await retries[region]?.();
  };

  return {
    recommendations, ranking, search, submitted, rankingDays, searchQuery, searchMode,
    loading, errors, loadRecommendations, loadRanking, runSearch, retry,
  };
};
