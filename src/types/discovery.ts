import type { BookGridItem } from "./book";
import type {
  BookSearchMode,
  DiscoveryList,
  RecommendBlock,
} from "../domain/discovery";

export type DiscoveryTab = "recommend" | "ranking" | "search";

export interface DiscoveryTabOption {
  readonly name: DiscoveryTab;
  readonly label: string;
  readonly icon?: string;
}

interface DiscoverySearchResultModel<T> {
  items: BookGridItem<T>[];
  pagination: { page: number; last: number } | null;
}

interface DiscoveryLoadState {
  loading: boolean;
  error: string;
}

export interface RecommendDiscoveryModel<T> extends DiscoveryLoadState {
  blocks: DiscoveryRecommendBlock<T>[];
}

interface DiscoveryRecommendBlock<T> {
  title: string;
  items: BookGridItem<T>[];
}

export interface RankingDiscoveryModel<T> extends DiscoveryLoadState {
  items: BookGridItem<T>[] | null;
  periods?: readonly DiscoveryRankingPeriod[];
  days?: number;
}

export interface SearchDiscoveryModel<T> extends DiscoveryLoadState {
  query: string;
  searchMode?: BookSearchMode;
  result: DiscoverySearchResultModel<T> | null;
}

export interface DiscoveryRankingPeriod {
  value: number;
  label: string;
}

/** Search-only state retained while navigating between results and a detail page. */
export interface DiscoverySearchCache<T> {
  search: DiscoveryList<T> | null;
  searchQuery: string;
  searchMode: BookSearchMode;
  scrollTop: number;
}

/** Service adapter contract used by the shared discovery state composable. */
export interface DiscoveryAdapter<T> {
  loadRecommendations(target: RecommendBlock<T>[]): Promise<void>;
  loadRanking(days: number): Promise<T[]>;
  search(
    query: string,
    page: number,
    mode: BookSearchMode,
  ): Promise<DiscoveryList<T>>;
}
