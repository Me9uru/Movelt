import type {
  DiscoveryRankingPeriod,
  DiscoveryTabOption,
} from "../../types/discovery";

export const discoveryTabs = [
  { name: "recommend", label: "推荐", icon: "star" },
  { name: "ranking", label: "排行榜", icon: "fire" },
  { name: "search", label: "搜索", icon: "magnify" },
] as const satisfies readonly DiscoveryTabOption[];

export const rankingPeriods = [
  { value: 7, label: "近 7 天" },
  { value: 30, label: "近 30 天" },
  { value: 365, label: "近一年" },
] as const satisfies readonly DiscoveryRankingPeriod[];
