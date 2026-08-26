import type { BookGridItem } from "./book";
import type { DiscoveryList, RecommendBlock } from "../domain/discovery";
import type { BookSearchMode } from "../domain/search";

export type DiscoveryTab = "recommend" | "ranking" | "search";
export type DiscoveryPrimaryTab = Exclude<DiscoveryTab, "search">;

export interface DiscoveryTabOption {
  name: DiscoveryPrimaryTab;
  label: string;
}

export interface DiscoveryTabProps<T extends string> {
  tabs: { name: T; label: string }[];
  modelValue: T;
  searchLabel: string;
  query: string;
  searchLoading: boolean;
  searchMode?: BookSearchMode;
  twoPrimary?: boolean;
  swipe?: boolean;
}

export interface BookSearchBarProps {
  modelValue: string;
  loading: boolean;
  placeholder?: string;
  ariaLabel?: string;
  searchMode?: BookSearchMode;
}

export interface DiscoveryViewSearchResult<T> {
  items: BookGridItem<T>[];
  pagination: { page: number; last: number } | null;
}

export interface RecommendDiscoveryConfig<T> {
  blocks: DiscoveryRecommendBlock<T>[];
  loading: boolean;
  error: string;
  emptyMessage?: string;
  errorTitle?: string;
}

export interface DiscoveryRecommendBlock<T> {
  title: string;
  items: BookGridItem<T>[];
}

export interface RankingDiscoveryConfig<T> {
  items: BookGridItem<T>[] | null;
  loading: boolean;
  error: string;
  periods?: { value: number; label: string }[];
  days?: number;
  emptyMessage?: string;
  errorTitle?: string;
}

export interface SearchDiscoveryConfig<T> {
  query: string;
  searchMode?: BookSearchMode;
  searchLabel?: string;
  result: DiscoveryViewSearchResult<T> | null;
  loading: boolean;
  error: string;
  promptMessage?: string;
  emptyMessage?: string;
  errorTitle?: string;
}

export interface DiscoveryViewProps<T> {
  tabs: DiscoveryTabOption[];
  modelValue: DiscoveryTab;
  recommend: RecommendDiscoveryConfig<T>;
  ranking: RankingDiscoveryConfig<T>;
  search: SearchDiscoveryConfig<T>;
  twoPrimary?: boolean;
  swipe?: boolean;
}

export interface RecommendDiscoveryProps<T> {
  blocks: DiscoveryRecommendBlock<T>[];
  loading: boolean;
  error: string;
  emptyMessage: string;
  errorTitle: string;
}

export interface RankingDiscoveryProps<T> {
  items: BookGridItem<T>[] | null;
  loading: boolean;
  error: string;
  emptyMessage: string;
  errorTitle: string;
  periods?: DiscoveryRankingPeriod[];
  days?: number;
}

export interface DiscoveryRankingPeriod {
  value: number;
  label: string;
}

export type DiscoveryRegion = DiscoveryTab;

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
