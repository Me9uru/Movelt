import { watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import type { DiscoveryAdapter, DiscoverySearchCache } from "../../types/discovery";
import { useDiscovery } from "./useDiscovery";
import { provideDiscoveryContext, type DiscoveryKind } from "./useDiscoveryContext";
import { useDiscoveryTabRoute } from "./useDiscoveryTabRoute";
import { useDiscoveryScroll } from "./useDiscoveryScroll";

/** Shared discovery navigation; adapters retain each domain's service behavior. */
export const useDiscoveryPage = <T>(
  kind: DiscoveryKind,
  adapter: DiscoveryAdapter<T>,
  cache: DiscoverySearchCache<T>,
) => {
  const route = useRoute();
  const router = useRouter();
  const routeName = kind === "novel" ? "novels" : "comic";
  const discovery = useDiscovery(adapter, cache);
  provideDiscoveryContext(kind);
  const { activeTab, selectTab } = useDiscoveryTabRoute(routeName);
  useDiscoveryScroll(cache, () => activeTab.value === "search");

  const syncQuery = (): boolean => {
    if (route.name !== routeName || typeof route.query.q !== "string") return false;
    const query = route.query.q;
    const mode = route.query.mode === "author" || route.query.mode === "tags" ? route.query.mode : "title";
    discovery.searchQuery.value = query;
    discovery.searchMode.value = mode;
    if (cache.submitted?.query !== query.trim() || cache.submitted.mode !== mode) {
      cache.search = null;
      cache.submitted = null;
      cache.scrollTop = 0;
      return true;
    }
    return false;
  };
  syncQuery();

  const ensureTab = (): void => {
    const tab = activeTab.value;
    if (discovery.loading.value[tab] || discovery.errors.value[tab]) return;
    if (tab === "recommend" && !discovery.recommendations.value.length) void discovery.loadRecommendations();
    if (tab === "ranking" && !discovery.ranking.value) void discovery.loadRanking();
    if (tab === "search" && !discovery.search.value && discovery.searchQuery.value.trim()) void discovery.runSearch();
  };

  // Query changes from back/forward navigation must update the search, too.
  let submitting = false;
  watch(() => [route.query.q, route.query.mode], () => {
    if (submitting) return;
    const changed = syncQuery();
    if (changed && activeTab.value === "search") void discovery.runSearch(1);
    else ensureTab();
  });
  watch(activeTab, () => { if (!submitting) ensureTab(); }, { immediate: true });

  const handleSearch = async (page: number): Promise<void> => {
    if (page > 1) {
      await discovery.runSearch(page);
      return;
    }
    const query = discovery.searchQuery.value.trim();
    if (!query) return;
    submitting = true;
    try {
      await router.replace({
        name: routeName,
        query: { ...route.query, tab: "search", q: query, mode: discovery.searchMode.value },
      });
    } finally {
      submitting = false;
    }
    await discovery.runSearch(1);
  };

  const detailQuery = () => ({
    ...(kind === "novel" ? { from: "novels" } : {}),
    ...(activeTab.value === "search" && discovery.submitted.value
      ? { tab: "search", q: discovery.submitted.value.query, mode: discovery.submitted.value.mode }
      : {}),
  });

  return { discovery, activeTab, selectTab, handleSearch, detailQuery };
};
