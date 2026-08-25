import type { BookGridItem } from "./book";
import type { DiscoveryList, RecommendBlock } from "../domain/discovery";
import type { BookSearchMode } from "../domain/search";

export type DiscoveryTab = "recommend" | "ranking" | "search";
export type DiscoveryPrimaryTab = Exclude<DiscoveryTab, "search">;

export interface DiscoveryTabOption {
  name: DiscoveryPrimaryTab;
  label: string;
}

export interface DiscoveryViewLabels {
  searchLabel?: string;
  recommendEmpty?: string;
  rankingEmpty?: string;
  searchPrompt?: string;
  searchEmpty?: string;
  errorTitles?: Partial<Record<DiscoveryTab, string>>;
}

export interface DiscoveryViewSearchResult<T> {
  items: BookGridItem<T>[];
  pagination: { page: number; last: number } | null;
}

export type DiscoveryRegion = DiscoveryTab;

/** Service adapter contract used by the shared discovery state composable. */
export interface DiscoveryAdapter<T> {
  loadRecommendations(target: RecommendBlock<T>[]): Promise<void>;
  loadRanking(days: number): Promise<T[]>;
  search(query: string, page: number, mode: BookSearchMode): Promise<DiscoveryList<T>>;
}
