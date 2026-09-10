/**
 * Domain-generic discovery DTOs shared by the novel and comic discovery flows.
 */

export interface RecommendBlock<T> {
  title: string;
  items: T[];
}

export type BookSearchMode = "title" | "author" | "tags";
export type BookOrder = "latest" | "view" | "new";
export type BookSearchRequestMode = BookSearchMode | "fuzzy" | "exact" | "name";

export interface DiscoveryList<T> {
  items: T[];
  pagination: {
    page: number;
    previous: number | null;
    next: number | null;
    first: number;
    last: number;
  };
}
