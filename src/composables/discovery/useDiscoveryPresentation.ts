import { computed } from "vue";
import type { BookGridItem } from "../../types/book";
import type {
  DiscoveryRankingPeriod,
  RankingDiscoveryModel,
  RecommendDiscoveryModel,
  SearchDiscoveryModel,
} from "../../types/discovery";
import type { DiscoveryState } from "./useDiscovery";

interface DiscoveryPresentationOptions {
  rankingPeriods?: readonly DiscoveryRankingPeriod[];
}

/** Maps shared discovery state into the discovery content components' props. */
export const useDiscoveryPresentation = <T>(
  discovery: DiscoveryState<T>,
  toBookItem: (item: T) => BookGridItem<T>,
  options: DiscoveryPresentationOptions = {},
) => {
  const searchResult = computed(() =>
    discovery.search.value
      ? {
        items: discovery.search.value.items.map(toBookItem),
        pagination: discovery.search.value.pagination,
      }
      : null,
  );
  const recommend = computed<RecommendDiscoveryModel<T>>(() => ({
    blocks: discovery.recommendations.value.map((block) => ({
      title: block.title,
      items: block.items.map(toBookItem),
    })),
    loading: discovery.loading.value.recommend,
    error: discovery.errors.value.recommend,
  }));
  const ranking = computed<RankingDiscoveryModel<T>>(() => ({
    items: discovery.ranking.value?.map(toBookItem) ?? null,
    loading: discovery.loading.value.ranking,
    error: discovery.errors.value.ranking,
    periods: options.rankingPeriods,
    days: options.rankingPeriods
      ? discovery.rankingDays.value
      : undefined,
  }));
  const search = computed<SearchDiscoveryModel<T>>(() => ({
    query: discovery.searchQuery.value,
    searchMode: discovery.searchMode.value,
    result: searchResult.value,
    loading: discovery.loading.value.search,
    error: discovery.errors.value.search,
  }));

  const retryDiscovery = (region: "recommend" | "ranking" | "search"): void => {
    void discovery.retry(region);
  };

  return { recommend, ranking, search, retryDiscovery };
};
