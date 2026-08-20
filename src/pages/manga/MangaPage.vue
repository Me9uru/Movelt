<script setup lang="ts">
import { Collection, Search } from "@element-plus/icons-vue";
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import BookSearchBar from "../../components/common/BookSearchBar.vue";
import LoadingOverlay from "../../components/common/LoadingOverlay.vue";
import { categoryPresets } from "../../composables/useDiscovery";
import { browseManga, type MangaBrowseType, type MangaSummary } from "../../services/manga";
import { getErrorMessage, showError } from "../../utils/error";
import ErrorState from "../../components/common/ErrorState.vue";

const router = useRouter();
const manga = ref<MangaSummary[]>([]);
const featured = ref<{ title: string; items: MangaSummary[] }[]>([]);
const query = ref("");
const categoryQuery = ref("");
const categoryTag = ref("");
const categorySearched = ref(false);
const loading = ref(false);
const error = ref("");
const visibleManga = computed(() => manga.value);
type MangaTab = "featured" | "ranking" | "category" | "search";
const activeTab = ref<MangaTab>("featured");
const searchDialogVisible = ref(false);
let allowSearchTab = false;

async function refresh(browseType: MangaBrowseType, term: string | null = null): Promise<void> {
  loading.value = true;
  error.value = "";
  try {
    manga.value = await browseManga(term, 1, browseType);
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
function selectCategory(tag: string): void {
  categoryTag.value = tag;
  categoryQuery.value = tag;
  categorySearched.value = true;
  void refresh("TAGS", tag);
}
function submitCategory(): void {
  const tags = categoryQuery.value.trim();
  if (!tags) return;
  categoryTag.value = "";
  categorySearched.value = true;
  void refresh("TAGS", tags);
}
function retry(): void {
  if (activeTab.value === "featured") { void refreshFeatured(); return; }
  if (activeTab.value === "ranking") { void refresh("POPULAR"); return; }
  if (activeTab.value === "category" && categorySearched.value) { void refresh("TAGS", categoryQuery.value.trim()); return; }
  if (activeTab.value === "search" && query.value.trim()) void refresh("SEARCH", query.value.trim());
}
function openSearchDialog(): void {
  searchDialogVisible.value = true;
}
function focusSearchInput(): void {
  const input = document.querySelector<HTMLInputElement>(".manga-search-dialog input");
  input?.focus();
}
function beforeTabLeave(nextName: string | number): boolean {
  if (nextName !== "search" || allowSearchTab) return true;
  openSearchDialog();
  return false;
}
function handleTabClick(pane: { paneName?: string | number }): void {
  if (pane.paneName === "search" && activeTab.value === "search") openSearchDialog();
}
async function submitSearch(): Promise<void> {
  if (!query.value.trim()) {
    void nextTick(focusSearchInput);
    return;
  }

  await refresh("SEARCH", query.value.trim());
  searchDialogVisible.value = false;
  allowSearchTab = true;
  activeTab.value = "search";
  void nextTick(() => {
    allowSearchTab = false;
  });
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
    <el-tabs v-model="activeTab" class="discovery-tabs manga-tabs" :before-leave="beforeTabLeave" @tab-click="handleTabClick">
      <el-tab-pane label="精选" name="featured" />
      <el-tab-pane label="排行榜" name="ranking" />
      <el-tab-pane label="分类" name="category" />
      <el-tab-pane name="search">
        <template #label><el-icon class="library-search-trigger discovery-search-trigger" aria-label="搜索漫画"><Search /></el-icon></template>
      </el-tab-pane>
    </el-tabs>

    <ErrorState v-if="error" title="漫画加载失败" :message="error" :loading="loading" @retry="retry" />

    <template v-else-if="activeTab === 'featured'">
      <LoadingOverlay v-if="loading" inline visible label="正在加载漫画" />
      <section v-for="block in featured" v-else :key="block.title" class="discovery-block">
        <div class="section-heading"><h2>{{ block.title }}</h2><el-tag class="count-tag" effect="plain">{{ block.items.length }} 本</el-tag></div>
        <div class="result-grid manga-grid">
          <el-card v-for="item in block.items" :key="item.id" class="book-card manga-card" shadow="hover" tabindex="0" @click="router.push({ name: 'manga-detail', params: { mangaId: item.id } })" @keydown.enter="router.push({ name: 'manga-detail', params: { mangaId: item.id } })">
            <el-image v-if="item.thumbnailUrl" class="book-cover" :src="item.thumbnailUrl" fit="cover" :alt="item.title"><template #error><span class="cover-placeholder"><el-icon><Collection /></el-icon></span></template></el-image>
            <span v-else class="cover-placeholder"><el-icon><Collection /></el-icon></span>
            <div class="book-meta manga-card-copy"><strong>{{ item.title }}</strong><span v-if="item.author">{{ item.author }}</span></div>
          </el-card>
        </div>
      </section>
    </template>

    <div v-else-if="activeTab === 'category'" class="manga-category">
      <div class="category-presets">
        <el-check-tag v-for="tag in categoryPresets" :key="tag" :checked="categoryTag === tag" @change="selectCategory(tag)">{{ tag }}</el-check-tag>
      </div>
      <div class="discovery-controls">
        <el-input v-model="categoryQuery" maxlength="40" clearable placeholder="输入标签，多个标签用逗号分隔" @keyup.enter="submitCategory" />
        <el-button type="primary" @click="submitCategory">查看分类</el-button>
      </div>
      <p v-if="!categorySearched" class="empty-tip">选择常用标签，或输入一个或多个标签开始筛选。</p>
      <LoadingOverlay v-else-if="loading" inline visible label="正在加载漫画" />
      <div v-else class="result-grid manga-grid">
        <el-card v-for="item in visibleManga" :key="item.id" class="book-card manga-card" shadow="hover" tabindex="0" @click="router.push({ name: 'manga-detail', params: { mangaId: item.id } })" @keydown.enter="router.push({ name: 'manga-detail', params: { mangaId: item.id } })">
          <el-image v-if="item.thumbnailUrl" class="book-cover" :src="item.thumbnailUrl" fit="cover" :alt="item.title"><template #error><span class="cover-placeholder"><el-icon><Collection /></el-icon></span></template></el-image>
          <span v-else class="cover-placeholder"><el-icon><Collection /></el-icon></span>
          <div class="book-meta manga-card-copy"><strong>{{ item.title }}</strong><span v-if="item.author">{{ item.author }}</span></div>
        </el-card>
      </div>
    </div>

    <LoadingOverlay v-else-if="loading" inline visible label="正在加载漫画" />
    <div v-else class="result-grid manga-grid">
      <el-card v-for="item in visibleManga" :key="item.id" class="book-card manga-card" shadow="hover" tabindex="0" @click="router.push({ name: 'manga-detail', params: { mangaId: item.id } })" @keydown.enter="router.push({ name: 'manga-detail', params: { mangaId: item.id } })">
        <el-image v-if="item.thumbnailUrl" class="book-cover" :src="item.thumbnailUrl" fit="cover" :alt="item.title"><template #error><span class="cover-placeholder"><el-icon><Collection /></el-icon></span></template></el-image>
        <span v-else class="cover-placeholder"><el-icon><Collection /></el-icon></span>
        <div class="book-meta manga-card-copy"><strong>{{ item.title }}</strong><span v-if="item.author">{{ item.author }}</span><em v-if="item.unreadCount">{{ item.unreadCount }} 话未读</em></div>
      </el-card>
    </div>
    <el-empty v-if="activeTab === 'category' && categorySearched && !loading && !error && visibleManga.length === 0" :image-size="108" description="这个分类还没有漫画" />
    <el-empty v-if="activeTab !== 'featured' && activeTab !== 'category' && !loading && !error && visibleManga.length === 0" :image-size="108" description="书库中暂无漫画" />

    <el-dialog
      v-model="searchDialogVisible"
      class="library-search-dialog manga-search-dialog"
      modal-class="library-search-mask"
      width="min(560px, calc(100vw - 32px))"
      align-center
      destroy-on-close
      :with-header="false"
      :show-close="false"
      @opened="focusSearchInput"
    >
      <BookSearchBar
        :model-value="query"
        :loading="loading"
        placeholder="输入漫画名称"
        aria-label="按漫画名称搜索"
        @update:model-value="query = $event"
        @submit="submitSearch"
      />
    </el-dialog>
  </section>
</template>
