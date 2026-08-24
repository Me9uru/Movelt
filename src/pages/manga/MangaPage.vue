<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import LoadingOverlay from "../../components/common/LoadingOverlay.vue";
import ContentTabs from "../../components/common/ContentTabs.vue";
import { browseManga, type MangaBrowseType } from "../../services/manga";
import type { MangaSummary } from "../../domain/content";
import type { BookSearchMode } from "../../domain/search";
import { getErrorMessage, showError } from "../../utils/error";
import ErrorState from "../../components/common/ErrorState.vue";
import MangaGrid from "../../components/manga/MangaGrid.vue";

const router = useRouter();
const manga = ref<MangaSummary[]>([]);
const featured = ref<{ title: string; items: MangaSummary[] }[]>([]);
const query = ref("");
const searchMode = ref<BookSearchMode>("title");
const loading = ref(false);
const error = ref("");
const visibleManga = computed(() => manga.value);
type MangaTab = "featured" | "ranking" | "search";
const mangaTabs: { name: Exclude<MangaTab, "search">; label: string }[] = [
  { name: "featured", label: "精选" },
  { name: "ranking", label: "排行榜" },
];
const activeTab = ref<MangaTab>("featured");

async function refresh(
  browseType: MangaBrowseType,
  term: string | null = null,
  mode?: BookSearchMode,
): Promise<void> {
  loading.value = true;
  error.value = "";
  try {
    manga.value = await browseManga(term, 1, browseType, mode);
  }
  catch (errorValue) { error.value = getErrorMessage(errorValue, "无法连接漫画服务器"); showError(errorValue, "无法连接漫画服务器"); }
  finally { loading.value = false; }
}
async function refreshFeatured(): Promise<void> {
  loading.value = true;
  error.value = "";
  try {
    const latest = await browseManga(null, 1, "LATEST");
    featured.value = [
      { title: "最近更新", items: latest.slice(0, 6) },
    ];
    loading.value = false;

    void Promise.allSettled([
      browseManga(null, 1, "POPULAR"),
      browseManga(null, 1, "NEW"),
    ]).then(([popular, newest]) => {
      if (popular.status === "fulfilled") {
        featured.value.push({ title: "热门作品", items: popular.value.slice(0, 6) });
      }
      if (newest.status === "fulfilled") {
        featured.value.push({ title: "新入库", items: newest.value.slice(0, 6) });
      }
    });
  } catch (errorValue) { error.value = getErrorMessage(errorValue, "无法连接漫画服务器"); showError(errorValue, "无法连接漫画服务器"); }
  finally { loading.value = false; }
}
function retry(): void {
  if (activeTab.value === "featured") { void refreshFeatured(); return; }
  if (activeTab.value === "ranking") { void refresh("POPULAR"); return; }
  if (activeTab.value === "search" && query.value.trim()) void refresh("SEARCH", query.value.trim(), searchMode.value);
}
function openManga(item: MangaSummary): void {
  void router.push({ name: "manga-detail", params: { mangaId: item.id } });
}
function handleSearch(): void {
  activeTab.value = "search";
  void refresh("SEARCH", query.value.trim(), searchMode.value);
}
watch(activeTab, (tab) => {
  if (tab === "featured" && featured.value.length === 0) void refreshFeatured();
  if (tab === "ranking") void refresh("POPULAR");
});
onMounted(async () => {
  await refreshFeatured();
});
</script>

<template>
  <section class="manga-view">
    <ContentTabs
      v-model="activeTab"
      :tabs="mangaTabs"
      search-label="搜索漫画"
      :query="query"
      :search-mode="searchMode"
      :search-loading="loading"
      two-primary
      @search="handleSearch"
      @update:query="query = $event"
      @update:search-mode="searchMode = $event"
    />

    <ErrorState v-if="error" title="漫画加载失败" :message="error" :loading="loading" @retry="retry" />

    <template v-else-if="activeTab === 'featured'">
      <LoadingOverlay v-if="loading" inline visible label="正在加载漫画" />
      <section v-for="block in featured" v-else :key="block.title" class="discovery-block">
        <div class="section-heading"><h2>{{ block.title }}</h2><var-chip class="count-tag" plain>{{ block.items.length }} 本</var-chip></div>
        <MangaGrid :manga="block.items" @open-manga="openManga" />
      </section>
    </template>

    <LoadingOverlay v-else-if="loading" inline visible label="正在加载漫画" />
    <MangaGrid v-else :manga="visibleManga" @open-manga="openManga" />
    <var-result v-if="activeTab !== 'featured' && !loading && !error && visibleManga.length === 0" :image-size="108" description="书库中暂无漫画" />

  </section>
</template>
