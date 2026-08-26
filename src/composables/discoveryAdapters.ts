import type { DiscoveryList } from "../domain/discovery";
import type { MangaSummary } from "../domain/manga";
import type { NovelSummary } from "../domain/novel";
import type { BookSearchMode } from "../domain/search";
import { browseManga } from "../services/manga";
import {
  getLatest,
  getRank,
  getRanking,
  searchDiscovery,
} from "../services/novel";
import type { DiscoveryAdapter } from "../types/discovery";

export const novelDiscoveryAdapter: DiscoveryAdapter<NovelSummary> = {
  async loadRecommendations(target) {
    const [latest, popular, newest] = await Promise.all([
      getLatest(),
      getRanking("view"),
      getRanking("new"),
    ]);
    target.push(
      { title: "最近更新", items: latest.items.slice(0, 6) },
      { title: "热门作品", items: popular.items.slice(0, 6) },
      { title: "新入库", items: newest.items.slice(0, 6) },
    );
  },
  async loadRanking(days) {
    return getRank(days);
  },
  async search(query: string, page: number, mode: BookSearchMode) {
    return searchDiscovery(query, page, mode);
  },
};

export const mangaDiscoveryAdapter: DiscoveryAdapter<MangaSummary> = {
  async loadRecommendations(target) {
    const latest = await browseManga(null, 1, "LATEST");
    target.push({ title: "最近更新", items: latest.slice(0, 6) });
    // target 是 composable 持有的 reactive 代理；在后台继续向同一数组 push，
    // 实现"最近更新先渲染、热门/新入库随后追加"的渐进加载。
    void Promise.allSettled([
      browseManga(null, 1, "POPULAR"),
      browseManga(null, 1, "NEW"),
    ]).then(([popular, newest]) => {
      if (popular.status === "fulfilled") {
        target.push({ title: "热门作品", items: popular.value.slice(0, 6) });
      }
      if (newest.status === "fulfilled") {
        target.push({ title: "新入库", items: newest.value.slice(0, 6) });
      }
    });
  },
  async loadRanking() {
    return browseManga(null, 1, "POPULAR");
  },
  async search(
    query: string,
    page: number,
    mode: BookSearchMode,
  ): Promise<DiscoveryList<MangaSummary>> {
    const items = await browseManga(query, page, "SEARCH", mode);
    // 上游 SearchComicSeries 固定每页 30 条且不返回总数：少于 30 条即末页。
    const last = items.length < 30 ? page : page + 1;
    return {
      items,
      pagination: { page, previous: null, next: null, first: 1, last },
    };
  },
};
