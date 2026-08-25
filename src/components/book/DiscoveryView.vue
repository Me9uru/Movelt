<script setup lang="ts" generic="T">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import BookGrid from "./BookGrid.vue";
import ContentTabs from "../common/ContentTabs.vue";
import ErrorState from "../common/ErrorState.vue";
import type { BookSearchMode } from "../../domain/search";
import type { BookGridItem } from "../../types/book";
import type {
  DiscoveryTab,
  DiscoveryTabOption,
  DiscoveryViewLabels,
  DiscoveryViewSearchResult,
} from "../../types/discovery";

const props = withDefaults(
  defineProps<{
    tabs: DiscoveryTabOption[];
    modelValue: DiscoveryTab;
    blocks: { title: string; items: BookGridItem<T>[] }[];
    ranking: BookGridItem<T>[] | null;
    searchResult: DiscoveryViewSearchResult<T> | null;
    loading: Record<DiscoveryTab, boolean>;
    errors: Record<DiscoveryTab, string>;
    query: string;
    searchMode?: BookSearchMode;
    rankingPeriods?: { value: number; label: string }[];
    rankingDays?: number;
    unavailableMessage?: string;
    labels?: DiscoveryViewLabels;
    twoPrimary?: boolean;
    swipe?: boolean;
  }>(),
  {
    searchMode: undefined,
    rankingPeriods: () => [],
    rankingDays: undefined,
    unavailableMessage: "",
    labels: () => ({}),
    twoPrimary: false,
    swipe: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: DiscoveryTab];
  "update:query": [value: string];
  "update:searchMode": [value: BookSearchMode];
  "update:rankingDays": [value: number];
  search: [page: number];
  open: [item: BookGridItem<T>];
  retryRecommend: [];
  retryRanking: [];
  retrySearch: [];
  initialize: [];
}>();

const labels = computed(() => {
  const custom = props.labels ?? {};
  return {
    searchLabel: custom.searchLabel ?? "搜索作品",
    recommendEmpty: custom.recommendEmpty ?? "暂无推荐内容",
    rankingEmpty: custom.rankingEmpty ?? "这个榜单还没有作品",
    searchPrompt: custom.searchPrompt ?? "输入作品名、作者或标签开始搜索",
    searchEmpty: custom.searchEmpty ?? "没有找到匹配的作品",
    errorTitles: {
      recommend: custom.errorTitles?.recommend ?? "推荐加载失败",
      ranking: custom.errorTitles?.ranking ?? "榜单加载失败",
      search: custom.errorTitles?.search ?? "搜索失败",
    },
  };
});

const searchSentinel = ref<HTMLElement | null>(null);
let loadMoreObserver: IntersectionObserver | undefined;

const hasMore = computed(() =>
  Boolean(
    props.searchResult?.pagination &&
    props.searchResult.pagination.page < props.searchResult.pagination.last,
  ),
);

onMounted(() => {
  loadMoreObserver = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        if (hasMore.value && !props.loading.search) {
          emit("search", (props.searchResult?.pagination?.page ?? 0) + 1);
        }
      }
    },
    { rootMargin: "400px 0px" },
  );
});

watch(searchSentinel, (element) => {
  loadMoreObserver?.disconnect();
  if (element) loadMoreObserver?.observe(element);
});

onBeforeUnmount(() => loadMoreObserver?.disconnect());
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
      <template #default
        ><var-button size="small" @click="emit('initialize')"
          >重新检查</var-button
        ></template
      >
    </var-alert>

    <template v-else>
      <ContentTabs
        :model-value="modelValue"
        :tabs="tabs"
        :search-label="labels.searchLabel"
        :query="query"
        :search-mode="searchMode"
        :search-loading="loading.search"
        :two-primary="twoPrimary"
        :swipe="swipe"
        @search="emit('search', 1)"
        @update:model-value="emit('update:modelValue', $event)"
        @update:query="emit('update:query', $event)"
        @update:search-mode="emit('update:searchMode', $event)"
      />

      <section v-if="modelValue === 'recommend'">
        <ErrorState
          v-if="errors.recommend"
          :title="labels.errorTitles.recommend"
          :message="errors.recommend"
          :loading="loading.recommend"
          @retry="emit('retryRecommend')"
        />
        <BookGrid
          v-else-if="loading.recommend"
          class="discovery-grid"
          :items="[]"
          loading
        />
        <var-result
          v-else-if="blocks.length === 0"
          type="empty"
          :description="labels.recommendEmpty"
        />
        <section
          v-for="block in blocks"
          v-else
          :key="block.title"
          class="discovery-block"
        >
          <div class="section-heading">
            <h2>{{ block.title }}</h2>
            <var-chip class="count-tag" plain
              >{{ block.items.length }} 本</var-chip
            >
          </div>
          <BookGrid
            class="discovery-grid"
            :items="block.items"
            @open="emit('open', $event)"
          />
        </section>
      </section>

      <section v-else-if="modelValue === 'ranking'">
        <var-tabs
          v-if="rankingPeriods.length"
          class="ranking-period-tabs"
          :active="String(rankingDays)"
          aria-label="榜单时间范围"
          @update:active="emit('update:rankingDays', Number($event))"
        >
          <var-tab
            v-for="period in rankingPeriods"
            :key="period.value"
            :name="String(period.value)"
            >{{ period.label }}</var-tab
          >
        </var-tabs>
        <ErrorState
          v-if="errors.ranking"
          :title="labels.errorTitles.ranking"
          :message="errors.ranking"
          :loading="loading.ranking"
          @retry="emit('retryRanking')"
        />
        <BookGrid
          v-else
          class="discovery-grid"
          :items="ranking || []"
          :loading="loading.ranking"
          @open="emit('open', $event)"
        />
        <var-result
          v-if="!loading.ranking && !errors.ranking && ranking?.length === 0"
          type="empty"
          :description="labels.rankingEmpty"
        />
      </section>

      <section v-else>
        <ErrorState
          v-if="errors.search"
          :title="labels.errorTitles.search"
          :message="errors.search"
          :loading="loading.search"
          @retry="emit('retrySearch')"
        />
        <BookGrid
          v-else-if="searchResult"
          class="discovery-grid"
          :items="searchResult.items"
          @open="emit('open', $event)"
        />
        <BookGrid
          v-else-if="loading.search"
          class="discovery-grid"
          :items="[]"
          loading
        />
        <var-result
          v-if="
            searchResult &&
            !loading.search &&
            !errors.search &&
            searchResult.items.length === 0
          "
          type="empty"
          :description="labels.searchEmpty"
        />
        <var-result
          v-else-if="!searchResult && !loading.search"
          type="empty"
          :description="labels.searchPrompt"
        />
        <div
          v-if="searchResult && hasMore"
          ref="searchSentinel"
          class="discovery-load-more"
        >
          <span v-if="loading.search">加载中…</span>
        </div>
      </section>
    </template>
  </section>
</template>
