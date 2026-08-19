<script setup lang="ts">
import { Collection, Search } from "@element-plus/icons-vue";
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import BookSearchBar from "../../components/common/BookSearchBar.vue";
import { browseManga, type MangaSummary } from "../../services/manga";

const router = useRouter();
const manga = ref<MangaSummary[]>([]);
const query = ref("");
const loading = ref(false);
const error = ref("");
const visibleManga = computed(() => manga.value);
type MangaTab = "popular" | "latest" | "search";
const activeTab = ref<MangaTab>("popular");
const searchDialogVisible = ref(false);
let allowSearchTab = false;

function describe(errorValue: unknown): string {
  if (typeof errorValue === "string") return errorValue;
  if (errorValue && typeof errorValue === "object" && "message" in errorValue && typeof errorValue.message === "string") return errorValue.message;
  return "无法连接漫画服务器";
}
async function refresh(browseType: "SEARCH" | "POPULAR" | "LATEST" = "POPULAR"): Promise<void> {
  loading.value = true;
  error.value = "";
  try {
    manga.value = await browseManga(browseType === "SEARCH" ? query.value.trim() : null, 1, browseType);
  }
  catch (errorValue) { error.value = describe(errorValue); }
  finally { loading.value = false; }
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

  await refresh("SEARCH");
  searchDialogVisible.value = false;
  allowSearchTab = true;
  activeTab.value = "search";
  void nextTick(() => {
    allowSearchTab = false;
  });
}
watch(activeTab, (tab) => {
  if (tab === "popular") void refresh("POPULAR");
  if (tab === "latest") void refresh("LATEST");
});
onMounted(async () => {
  await refresh();
});
</script>

<template>
  <section class="manga-view">
    <el-alert v-if="error" type="error" show-icon :closable="false" :title="error" />

    <el-tabs v-model="activeTab" class="discovery-tabs manga-tabs" :before-leave="beforeTabLeave" @tab-click="handleTabClick">
      <el-tab-pane label="热门" name="popular" />
      <el-tab-pane label="最新" name="latest" />
      <el-tab-pane name="search">
        <template #label><el-icon class="library-search-trigger discovery-search-trigger" aria-label="搜索漫画"><Search /></el-icon></template>
      </el-tab-pane>
    </el-tabs>

    <div v-loading="loading" class="manga-grid">
      <button v-for="item in visibleManga" :key="item.id" class="manga-card" type="button" @click="router.push({ name: 'manga-detail', params: { mangaId: item.id } })">
        <el-image v-if="item.thumbnailUrl" :src="item.thumbnailUrl" fit="cover" :alt="item.title"><template #error><span class="manga-cover-fallback"><el-icon><Collection /></el-icon></span></template></el-image>
        <span v-else class="manga-cover-fallback"><el-icon><Collection /></el-icon></span>
        <span class="manga-card-copy"><strong>{{ item.title }}</strong><small>{{ item.author || item.sourceName || "未知来源" }}</small><em v-if="item.unreadCount">{{ item.unreadCount }} 话未读</em></span>
      </button>
    </div>
    <el-empty v-if="!loading && !error && visibleManga.length === 0" :image-size="108" description="书库中暂无漫画" />

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
