<script setup lang="ts">
import { ref, watch } from "vue";
import { rankingPeriods } from "../../composables/useDiscovery";
import type { DiscoveryList, NovelSummary, RecommendBlock } from "../../domain/content";
import type { BookSearchMode } from "../../domain/search";
import BookGrid from "../../components/novel/BookGrid.vue";
import ContentTabs from "../../components/common/ContentTabs.vue";
import ErrorState from "../../components/common/ErrorState.vue";

const props = defineProps<{
  unavailableMessage: string;
  recommendations: RecommendBlock[];
  ranking: NovelSummary[] | null;
  searchResult: DiscoveryList | null;
  rankingDays: number;
  searchQuery: string;
  searchMode: BookSearchMode;
  loading: Record<"recommend" | "ranking" | "search", boolean>;
  errors: Record<"recommend" | "ranking" | "search", string>;
}>();

const emit = defineEmits<{
  initialize: [];
  retryRecommendations: [];
  loadRanking: [days?: number];
  search: [page: number];
  openNovel: [novel: NovelSummary];
  "update:rankingDays": [value: number];
  "update:searchQuery": [value: string];
  "update:searchMode": [value: BookSearchMode];
}>();

type DiscoveryTab = "recommend" | "ranking" | "search";

const discoveryTabs: { name: Exclude<DiscoveryTab, "search">; label: string }[] = [
  { name: "recommend", label: "推荐" },
  { name: "ranking", label: "排行榜" },
];
const activeTab = ref<DiscoveryTab>("recommend");

function handleSearch(): void {
  emit("search", 1);
  activeTab.value = "search";
}

function selectRankingPeriod(days: number) {
  if (days === props.rankingDays) return;

  emit("update:rankingDays", days);
  emit("loadRanking", days);
}

watch(activeTab, (tab) => {
  if (tab === "recommend" && props.recommendations.length === 0 && !props.loading.recommend && !props.errors.recommend) {
    emit("retryRecommendations");
  }
  if (tab === "ranking" && !props.ranking && !props.loading.ranking && !props.errors.ranking) {
    emit("loadRanking");
  }
}, { immediate: true });
</script>

<template>
  <section class="discovery-view">
    <var-alert
      v-if="unavailableMessage"
      class="service-alert"
      :title="unavailableMessage"
      type="warning"
      :closeable="false"
    >
      <template #default><var-button size="small" @click="emit('initialize')">重新检查</var-button></template>
    </var-alert>

    <template v-else>
      <ContentTabs
        v-model="activeTab"
        :tabs="discoveryTabs"
        search-label="搜索作品"
        :query="searchQuery"
        :search-mode="searchMode"
        :search-loading="loading.search"
        two-primary
        swipe
        @search="handleSearch"
        @update:query="emit('update:searchQuery', $event)"
        @update:search-mode="emit('update:searchMode', $event)"
      />

    <section v-if="activeTab === 'recommend'">
      <ErrorState v-if="errors.recommend" title="推荐加载失败" :message="errors.recommend" :loading="loading.recommend" @retry="emit('retryRecommendations')" />
      <BookGrid v-else-if="loading.recommend" :books="[]" loading />
      <var-result v-else-if="recommendations.length === 0" description="暂无推荐内容" />
      <section v-for="block in recommendations" v-else :key="block.title" class="discovery-block">
        <div class="section-heading"><h2>{{ block.title }}</h2><var-chip class="count-tag" plain>{{ block.items.length }} 本</var-chip></div>
        <BookGrid :books="block.items" @open-novel="emit('openNovel', $event)" />
      </section>
    </section>

    <section v-else-if="activeTab === 'ranking'">
      <var-tabs
          class="ranking-period-tabs"
          :active="String(rankingDays)"
          aria-label="榜单时间范围"
          @update:active="selectRankingPeriod(Number($event))"
        >
          <var-tab
            v-for="period in rankingPeriods"
            :key="period.value"
            :name="String(period.value)"
          >{{ period.label }}</var-tab>
      </var-tabs>
      <ErrorState v-if="errors.ranking" title="榜单加载失败" :message="errors.ranking" :loading="loading.ranking" @retry="emit('loadRanking')" />
      <BookGrid v-else :books="ranking || []" :loading="loading.ranking" @open-novel="emit('openNovel', $event)" />
      <var-result v-if="!loading.ranking && !errors.ranking && ranking?.length === 0" description="这个榜单还没有作品" />
    </section>

    <section v-else>
      <ErrorState v-if="errors.search" title="搜索失败" :message="errors.search" :loading="loading.search" @retry="emit('search', searchResult?.pagination.page || 1)" />
      <BookGrid v-else-if="searchResult || loading.search" :books="searchResult?.items || []" :loading="loading.search" @open-novel="emit('openNovel', $event)" />
      <var-result v-if="searchResult && !loading.search && !errors.search && searchResult.items.length === 0" description="没有找到匹配的作品" />
      <var-result v-else-if="!searchResult && !loading.search" description="输入作品名、作者或标签开始搜索" />
      <var-pagination v-if="searchResult && searchResult.pagination.last > 1" class="discovery-pagination" :size="1" :total="searchResult.pagination.last"
        :current="searchResult.pagination.page" @update:current="emit('search', Number($event))" />
    </section>
    </template>

  </section>
</template>
