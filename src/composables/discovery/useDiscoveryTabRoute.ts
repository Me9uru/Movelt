import { ref, watch, type Ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import type { DiscoveryTab } from "../../types/discovery";
import { discoveryTabs } from "./config";

type DiscoveryRouteName = "novels" | "comic";

interface DiscoveryTabRoute {
  activeTab: Ref<DiscoveryTab>;
  selectTab: (tab: DiscoveryTab) => void;
}

const getDiscoveryTab = (value: unknown): DiscoveryTab => {
  const tab = Array.isArray(value) ? value[0] : value;
  return discoveryTabs.some((option) => option.name === tab)
    ? (tab as DiscoveryTab)
    : "recommend";
};

export const useDiscoveryTabRoute = (
  routeName: DiscoveryRouteName,
): DiscoveryTabRoute => {
  const route = useRoute();
  const router = useRouter();
  const activeTab = ref<DiscoveryTab>(getDiscoveryTab(route.query.tab));

  const selectTab = (tab: DiscoveryTab): void => {
    activeTab.value = tab;
    const query = { ...route.query };
    if (tab === "recommend") delete query.tab;
    else query.tab = tab;
    if (tab !== "search") {
      delete query.q;
      delete query.mode;
    }
    void router.replace({ name: routeName, query });
  };

  watch(() => route.query.tab, (tab) => {
    activeTab.value = getDiscoveryTab(tab);
  });

  return { activeTab, selectTab };
};
