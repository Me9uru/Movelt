import { computed } from "vue";
import type { BookGridItem } from "../types/book";
import type {
  RankingDiscoveryConfig,
  RecommendDiscoveryConfig,
  SearchDiscoveryConfig,
} from "../types/discovery";
import type { DiscoveryState } from "./useDiscovery";

type RecommendMessages<T> = Pick<
  RecommendDiscoveryConfig<T>,
  "emptyMessage" | "errorTitle"
>;
type RankingMessages<T> = Pick<
  RankingDiscoveryConfig<T>,
  "periods" | "emptyMessage" | "errorTitle"
>;
type SearchMessages<T> = Pick<
  SearchDiscoveryConfig<T>,
  "searchLabel" | "promptMessage" | "emptyMessage" | "errorTitle"
>;

interface DiscoveryPresentationOptions<T> {
  recommend?: RecommendMessages<T>;
  ranking?: RankingMessages<T>;
  search?: SearchMessages<T>;
}

/** Maps shared discovery state into the presentation contracts of DiscoveryView. */
export function useDiscoveryPresentation<T>(
  discovery: DiscoveryState<T>,
  toBookItem: (item: T) => BookGridItem<T>,
  options: DiscoveryPresentationOptions<T> = {},
) {
  const searchResult = computed(() =>
    discovery.search.value
      ? {
          items: discovery.search.value.items.map(toBookItem),
          pagination: discovery.search.value.pagination,
        }
      : null,
  );
  const recommend = computed<RecommendDiscoveryConfig<T>>(() => ({
    blocks: discovery.recommendations.value.map((block) => ({
      title: block.title,
      items: block.items.map(toBookItem),
    })),
    loading: discovery.loading.value.recommend,
    error: discovery.errors.value.recommend,
    ...options.recommend,
  }));
  const ranking = computed<RankingDiscoveryConfig<T>>(() => ({
    items: discovery.ranking.value?.map(toBookItem) ?? null,
    loading: discovery.loading.value.ranking,
    error: discovery.errors.value.ranking,
    ...options.ranking,
    days: options.ranking?.periods
      ? discovery.rankingDays.value
      : undefined,
  }));
  const search = computed<SearchDiscoveryConfig<T>>(() => ({
    query: discovery.searchQuery.value,
    searchMode: discovery.searchMode.value,
    result: searchResult.value,
    loading: discovery.loading.value.search,
    error: discovery.errors.value.search,
    ...options.search,
  }));

  function retryDiscovery(region: "recommend" | "ranking" | "search"): void {
    if (region === "recommend") void discovery.loadRecommendations();
    if (region === "ranking") void discovery.loadRanking();
    if (region === "search") {
      void discovery.runSearch(searchResult.value?.pagination?.page || 1);
    }
  }

  return { recommend, ranking, search, retryDiscovery };
}
