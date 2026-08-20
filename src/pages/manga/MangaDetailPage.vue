<script setup lang="ts">
import { ArrowLeft, Collection, Reading, VideoPlay } from "@element-plus/icons-vue";
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { addToMangaBookshelf, getManga, isOnMangaBookshelf, removeFromMangaBookshelf, type MangaDetail } from "../../services/manga";
import { getErrorMessage, showError } from "../../utils/error";
import ErrorState from "../../components/common/ErrorState.vue";
import WorkDescription from "../../components/common/WorkDescription.vue";

const route = useRoute(); const router = useRouter();
const manga = ref<MangaDetail | null>(null); const loading = ref(true); const error = ref("");
const mangaId = computed(() => String(route.params.mangaId));
const onBookshelf = ref(false);
const resumeChapterId = computed(() => {
  const chapterId = manga.value?.readPosition?.chapterId;
  return manga.value?.chapters.some((chapter) => chapter.id === chapterId) ? chapterId : null;
});
async function load(): Promise<void> {
  loading.value = true;
  error.value = "";
  try {
    const detail = await getManga(mangaId.value);
    manga.value = detail;
    onBookshelf.value = await isOnMangaBookshelf(mangaId.value);
  } catch (value) { error.value = getErrorMessage(value, "无法加载漫画详情"); showError(value, "无法加载漫画详情"); }
  finally { loading.value = false; }
}
onMounted(() => void load());
function read(chapterId: string): void { void router.push({ name: "manga-reader", params: { mangaId: mangaId.value, chapterId } }); }
async function toggleBookshelf(): Promise<void> {
  try {
    if (onBookshelf.value) await removeFromMangaBookshelf(mangaId.value);
    else await addToMangaBookshelf(mangaId.value);
    onBookshelf.value = !onBookshelf.value;
  } catch (errorValue) {
    showError(errorValue, "更新漫画书架失败");
  }
}
</script>
<template>
  <section class="manga-detail-view" v-loading="loading">
    <el-button :icon="ArrowLeft" text @click="router.push({ name: 'manga' })">返回漫画</el-button>
    <ErrorState v-if="error" title="漫画详情加载失败" :message="error" :loading="loading" @retry="load" />
    <template v-else-if="manga">
      <article class="manga-profile">
        <el-image v-if="manga.thumbnailUrl" class="manga-detail-cover" :src="manga.thumbnailUrl" fit="cover"><template #error><span class="manga-cover-fallback"><el-icon><Collection /></el-icon></span></template></el-image>
        <div><h1>{{ manga.title }}</h1><p class="manga-byline">{{ manga.author || manga.artist || "作者未知" }} · {{ manga.status }}</p><WorkDescription class="manga-description" :content="manga.description" /><div class="manga-tags"><el-tag v-for="tag in manga.genre" :key="tag" effect="plain">{{ tag }}</el-tag></div></div>
      </article>
      <section class="manga-chapter-section"><header><div><h2>章节</h2><p>{{ manga.chapters.length }} 话</p></div><div><el-button @click="toggleBookshelf">{{ onBookshelf ? "已收藏" : "收藏漫画" }}</el-button><el-button v-if="resumeChapterId" type="primary" :icon="VideoPlay" @click="read(resumeChapterId)">继续阅读</el-button><el-button v-else-if="manga.chapters[0]" type="primary" :icon="VideoPlay" @click="read(manga.chapters[0].id)">开始阅读</el-button></div></header>
        <button v-for="chapter in manga.chapters" :key="chapter.id" class="manga-chapter" type="button" @click="read(chapter.id)"><el-icon><Reading /></el-icon><span><strong>{{ chapter.name || `第 ${chapter.chapterNumber} 话` }}</strong><small v-if="chapter.pageCount">{{ chapter.pageCount }} 页</small></span><el-tag v-if="!chapter.isRead" size="small">未读</el-tag></button>
      </section>
    </template>
  </section>
</template>
