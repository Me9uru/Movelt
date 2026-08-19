<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { Search } from "@element-plus/icons-vue";
import { categoryPresets, rankingSorts } from "../../composables/useDiscovery";
import type {
  DiscoveryList, NovelSummary, RankingSort, RecommendBlock,
} from "../../services/novel";
import BookGrid from "../../components/library/BookGrid.vue";
import BookSearchBar from "../../components/common/BookSearchBar.vue";

const props = defineProps<{
  unavailableMessage: string;
  recommendations: RecommendBlock[];
  ranking: DiscoveryList | null;
  category: DiscoveryList | null;
  searchResult: DiscoveryList | null;
  rankingSort: RankingSort;
  categoryTag: string;
  customTag: string;
  searchQuery: string;
  loading: Record<"recommend" | "ranking" | "category" | "search", boolean>;
  errors: Record<"recommend" | "ranking" | "category" | "search", string>;
}>();

const emit = defineEmits<{
  initialize: [];
  retryRecommendations: [];
  loadRanking: [page: number];
  loadCategory: [page: number];
  search: [page: number];
  selectCategory: [tag: string];
  openNovel: [novel: NovelSummary];
  "update:rankingSort": [value: RankingSort];
  "update:customTag": [value: string];
  "update:searchQuery": [value: string];
}>();

type DiscoveryTab = "recommend" | "ranking" | "category" | "search";

const discoveryTabs: DiscoveryTab[] = ["recommend", "ranking", "category", "search"];
const activeTab = ref<DiscoveryTab>("recommend");
const searchDialogVisible = ref(false);
let allowSearchTab = false;
let touchStart: { x: number; y: number } | null = null;

function openSearchDialog() {
  searchDialogVisible.value = true;
}

function focusSearchInput() {
  const input = document.querySelector<HTMLInputElement>(".discovery-search-dialog input");
  input?.focus();
}

function beforeTabLeave(nextName: string | number) {
  if (nextName !== "search" || allowSearchTab) return true;

  openSearchDialog();
  return false;
}

function handleTabClick(pane: { paneName?: string | number }) {
  if (pane.paneName === "search" && activeTab.value === "search") openSearchDialog();
}

function submitSearch() {
  if (!props.searchQuery.trim()) {
    void nextTick(focusSearchInput);
    return;
  }

  emit("search", 1);
  searchDialogVisible.value = false;
  allowSearchTab = true;
  activeTab.value = "search";
  void nextTick(() => {
    allowSearchTab = false;
  });
}

function handleTouchStart(event: TouchEvent) {
  if (event.touches.length !== 1) {
    touchStart = null;
    return;
  }

  const touch = event.touches[0];
  touchStart = { x: touch.clientX, y: touch.clientY };
}

function handleTouchEnd(event: TouchEvent) {
  if (!touchStart || event.changedTouches.length !== 1) {
    touchStart = null;
    return;
  }

  const touch = event.changedTouches[0];
  const deltaX = touch.clientX - touchStart.x;
  const deltaY = touch.clientY - touchStart.y;
  touchStart = null;

  if (Math.abs(deltaX) < 50 || Math.abs(deltaX) <= Math.abs(deltaY) * 1.25) return;

  const currentIndex = discoveryTabs.indexOf(activeTab.value);
  const nextIndex = deltaX < 0 ? currentIndex + 1 : currentIndex - 1;
  const nextTab = discoveryTabs[nextIndex];
  if (nextTab) activeTab.value = nextTab;
}

watch(activeTab, (tab) => {
  if (tab === "ranking" && !props.ranking && !props.loading.ranking && !props.errors.ranking) {
    emit("loadRanking", 1);
  } else if (tab === "category" && !props.category && !props.loading.category && !props.errors.category) {
    emit("loadCategory", 1);
  }
});
</script>

<template>
  <section class="discovery-view">
    <el-alert
      v-if="unavailableMessage"
      class="service-alert"
      :title="unavailableMessage"
      type="warning"
      show-icon
      :closable="false"
    >
      <template #default><el-button size="small" @click="emit('initialize')">重新检查</el-button></template>
    </el-alert>

    <el-tabs
      v-else
      v-model="activeTab"
      class="discovery-tabs"
      :before-leave="beforeTabLeave"
      @tab-click="handleTabClick"
      @touchstart.passive="handleTouchStart"
      @touchend.passive="handleTouchEnd"
      @touchcancel="touchStart = null"
    >
      <el-tab-pane label="推荐" name="recommend" lazy>
        <div v-if="errors.recommend" class="region-state">
          <el-result icon="error" title="推荐加载失败" :sub-title="errors.recommend">
            <template #extra><el-button @click="emit('retryRecommendations')">重试</el-button></template>
          </el-result>
        </div>
        <BookGrid v-else-if="loading.recommend" :books="[]" loading />
        <el-empty v-else-if="recommendations.length === 0" description="暂无推荐内容" />
        <section v-for="block in recommendations" v-else :key="block.title" class="discovery-block">
          <div class="section-heading"><h2>{{ block.title }}</h2><el-tag class="count-tag" effect="plain">{{ block.items.length }} 本</el-tag></div>
          <BookGrid :books="block.items" @open-novel="emit('openNovel', $event)" />
        </section>
      </el-tab-pane>

      <el-tab-pane label="排行榜" name="ranking" lazy>
        <div class="discovery-controls">
          <el-select
            :model-value="rankingSort"
            aria-label="榜单排序"
            @update:model-value="emit('update:rankingSort', $event); emit('loadRanking', 1)"
          >
            <el-option v-for="sort in rankingSorts" :key="sort.value" :label="sort.label" :value="sort.value" />
          </el-select>
        </div>
        <el-result v-if="errors.ranking" icon="error" title="榜单加载失败" :sub-title="errors.ranking">
          <template #extra><el-button @click="emit('loadRanking', ranking?.pagination.page || 1)">重试</el-button></template>
        </el-result>
        <BookGrid v-else :books="ranking?.items || []" :loading="loading.ranking" @open-novel="emit('openNovel', $event)" />
        <el-empty v-if="!loading.ranking && !errors.ranking && ranking?.items.length === 0" description="这个榜单还没有作品" />
        <el-pagination v-if="ranking && ranking.pagination.last > 1" class="discovery-pagination" background layout="prev, pager, next"
          :current-page="ranking.pagination.page" :page-count="ranking.pagination.last" @current-change="emit('loadRanking', $event)" />
      </el-tab-pane>

      <el-tab-pane label="分类" name="category" lazy>
        <div class="category-presets">
          <el-check-tag v-for="tag in categoryPresets" :key="tag" :checked="!customTag && categoryTag === tag" @change="emit('selectCategory', tag)">{{ tag }}</el-check-tag>
        </div>
        <div class="discovery-controls">
          <el-input :model-value="customTag" maxlength="40" clearable placeholder="自定义标签"
            @update:model-value="emit('update:customTag', $event)" @keyup.enter="emit('loadCategory', 1)" />
          <el-button type="primary" @click="emit('loadCategory', 1)">查看分类</el-button>
        </div>
        <el-result v-if="errors.category" icon="error" title="分类加载失败" :sub-title="errors.category">
          <template #extra><el-button @click="emit('loadCategory', category?.pagination.page || 1)">重试</el-button></template>
        </el-result>
        <BookGrid v-else :books="category?.items || []" :loading="loading.category" @open-novel="emit('openNovel', $event)" />
        <el-empty v-if="!loading.category && !errors.category && category?.items.length === 0" description="这个分类还没有作品" />
        <el-pagination v-if="category && category.pagination.last > 1" class="discovery-pagination" background layout="prev, pager, next"
          :current-page="category.pagination.page" :page-count="category.pagination.last" @current-change="emit('loadCategory', $event)" />
      </el-tab-pane>

      <el-tab-pane name="search" lazy>
        <template #label>
          <el-icon class="library-search-trigger discovery-search-trigger" aria-label="搜索作品"><Search /></el-icon>
        </template>
        <el-result v-if="errors.search" icon="error" title="搜索失败" :sub-title="errors.search">
          <template #extra><el-button @click="emit('search', searchResult?.pagination.page || 1)">重试</el-button></template>
        </el-result>
        <BookGrid v-else-if="searchResult || loading.search" :books="searchResult?.items || []" :loading="loading.search" @open-novel="emit('openNovel', $event)" />
        <el-empty v-if="searchResult && !loading.search && !errors.search && searchResult.items.length === 0" description="没有找到匹配的作品" />
        <el-empty v-else-if="!searchResult && !loading.search" description="输入书名开始搜索" />
        <el-pagination v-if="searchResult && searchResult.pagination.last > 1" class="discovery-pagination" background layout="prev, pager, next"
          :current-page="searchResult.pagination.page" :page-count="searchResult.pagination.last" @current-change="emit('search', $event)" />
      </el-tab-pane>
    </el-tabs>

    <el-dialog
      v-model="searchDialogVisible"
      class="library-search-dialog discovery-search-dialog"
      modal-class="library-search-mask"
      width="min(560px, calc(100vw - 32px))"
      align-center
      destroy-on-close
      :with-header="false"
      :show-close="false"
      @opened="focusSearchInput"
    >
      <BookSearchBar
        class="discovery-search"
        :model-value="searchQuery"
        :loading="loading.search"
        @update:model-value="emit('update:searchQuery', $event)"
        @submit="submitSearch"
      />
    </el-dialog>
  </section>
</template>
