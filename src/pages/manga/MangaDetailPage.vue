<script setup lang="ts">
import { ArrowLeft, Collection, Reading, VideoPlay } from "@element-plus/icons-vue";
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { getManga, type MangaDetail } from "../../services/manga";

const route = useRoute(); const router = useRouter();
const manga = ref<MangaDetail | null>(null); const loading = ref(true); const error = ref("");
const mangaId = computed(() => String(route.params.mangaId));
onMounted(async () => {
  try {
    const detail = await getManga(mangaId.value);
    manga.value = detail;
  } catch (value) { error.value = typeof value === "string" ? value : "无法加载漫画详情"; }
  finally { loading.value = false; }
});
function read(chapterId: string): void { void router.push({ name: "manga-reader", params: { mangaId: mangaId.value, chapterId } }); }
</script>
<template>
  <section class="manga-detail-view" v-loading="loading">
    <el-button :icon="ArrowLeft" text @click="router.push({ name: 'manga' })">返回漫画</el-button>
    <el-alert v-if="error" type="error" :title="error" show-icon :closable="false" />
    <template v-else-if="manga">
      <article class="manga-profile">
        <el-image v-if="manga.thumbnailUrl" class="manga-detail-cover" :src="manga.thumbnailUrl" fit="cover"><template #error><span class="manga-cover-fallback"><el-icon><Collection /></el-icon></span></template></el-image>
        <div><p class="eyebrow">{{ manga.sourceName || "lnovelApi" }}</p><h1>{{ manga.title }}</h1><p class="manga-byline">{{ manga.author || manga.artist || "作者未知" }} · {{ manga.status }}</p><p class="manga-description">{{ manga.description || "暂无简介。" }}</p><div class="manga-tags"><el-tag v-for="tag in manga.genre" :key="tag" effect="plain">{{ tag }}</el-tag></div></div>
      </article>
      <section class="manga-chapter-section"><header><div><h2>章节</h2><p>{{ manga.chapters.length }} 话</p></div><el-button v-if="manga.chapters[0]" type="primary" :icon="VideoPlay" @click="read(manga.chapters[0].id)">开始阅读</el-button></header>
        <button v-for="chapter in manga.chapters" :key="chapter.id" class="manga-chapter" type="button" @click="read(chapter.id)"><el-icon><Reading /></el-icon><span><strong>{{ chapter.name || `第 ${chapter.chapterNumber} 话` }}</strong><small v-if="chapter.pageCount">{{ chapter.pageCount }} 页</small></span><el-tag v-if="!chapter.isRead" size="small">未读</el-tag></button>
      </section>
    </template>
  </section>
</template>
