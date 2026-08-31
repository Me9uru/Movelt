import { reactive } from "vue";
import { defineStore } from "pinia";
import type { ComicSummary } from "../domain/comic";
import type { NovelSummary } from "../domain/novel";
import type { DiscoverySearchCache } from "../types/discovery";

const createCache = <T>(): DiscoverySearchCache<T> => ({
  search: null,
  searchQuery: "",
  searchMode: "title",
  scrollTop: 0,
});

const resetCache = <T>(cache: DiscoverySearchCache<T>): void => {
  Object.assign(cache, createCache<T>());
};

/** Search results kept while navigating to and from a book detail page. */
export const useDiscoverySearchStore = defineStore("discovery-search", () => {
  const novel = reactive(createCache<NovelSummary>());
  const comic = reactive(createCache<ComicSummary>());

  const clear = (): void => {
    resetCache(novel);
    resetCache(comic);
  };

  return { novel, comic, clear };
});
