/**
 * Domain-generic discovery DTOs shared by the novel and manga discovery flows.
 */

export interface RecommendBlock<T> {
  title: string;
  items: T[];
}

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
