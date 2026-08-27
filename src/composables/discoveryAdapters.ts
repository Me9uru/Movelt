import type { DiscoveryList } from "../domain/discovery";
import type { ComicSummary } from "../domain/comic";
import type { NovelSummary } from "../domain/novel";
import type { BookSearchMode } from "../domain/search";
import { listComic, searchComic } from "../services/comic";
import {
  listNovel,
  rankNovels,
  searchDiscovery,
} from "../services/novel";
import type { DiscoveryAdapter } from "../types/discovery";

const RECOMMENDATION_PAGE_SIZE = 6;
const DISCOVERY_PAGE_SIZE = 24;

export const novelDiscoveryAdapter: DiscoveryAdapter<NovelSummary> = {
  async loadRecommendations(target) {
    const [latest, popular, newest] = await Promise.all([
      listNovel(1, RECOMMENDATION_PAGE_SIZE, "latest"),
      listNovel(1, RECOMMENDATION_PAGE_SIZE, "view"),
      listNovel(1, RECOMMENDATION_PAGE_SIZE, "new"),
    ]);
    target.push(
      { title: "最近更新", items: latest },
      { title: "热门作品", items: popular },
      { title: "新入库", items: newest },
    );
  },
  async loadRanking(days) {
    return rankNovels(days);
  },
  async search(query: string, page: number, mode: BookSearchMode) {
    // 小说 UI 的“作品名”搜索对应官方的系列名/作品名模式，而不是卷标题模式。
    return searchDiscovery(
      query,
      page,
      DISCOVERY_PAGE_SIZE,
      mode === "title" ? "name" : mode,
    );
  },
};

export const comicDiscoveryAdapter: DiscoveryAdapter<ComicSummary> = {
  async loadRecommendations(target) {
    const [latest, popular, newest] = await Promise.allSettled([
      listComic(1, RECOMMENDATION_PAGE_SIZE, "latest"),
      listComic(1, RECOMMENDATION_PAGE_SIZE, "view"),
      listComic(1, RECOMMENDATION_PAGE_SIZE, "new"),
    ]);

    if (latest.status === "rejected") throw latest.reason;

    target.push({ title: "最近更新", items: latest.value });
    if (popular.status === "fulfilled") {
      target.push({ title: "热门作品", items: popular.value });
    }
    if (newest.status === "fulfilled") {
      target.push({ title: "新入库", items: newest.value });
    }
  },
  async loadRanking() {
    return listComic(1, DISCOVERY_PAGE_SIZE, "view");
  },
  async search(
    query: string,
    page: number,
    mode: BookSearchMode,
  ): Promise<DiscoveryList<ComicSummary>> {
    return searchComic(query, page, DISCOVERY_PAGE_SIZE, mode);
  },
};
