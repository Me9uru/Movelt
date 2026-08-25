<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  addToMangaBookshelf,
  getManga,
  isOnMangaBookshelf,
  removeFromMangaBookshelf,
} from "../../services/manga";
import type { MangaChapter, MangaDetail } from "../../domain/manga";
import { getErrorMessage, showError } from "../../utils/error";
import BookChapterList from "../../components/book/BookChapterList.vue";
import type { BookChapterItem } from "../../types/book";
import BookDetailView from "../../components/book/BookDetailView.vue";
import BookDetailSection from "../../components/book/BookDetailSection.vue";

const route = useRoute();
const router = useRouter();
const manga = ref<MangaDetail | null>(null);
const loading = ref(true);
const error = ref("");
const mangaId = computed(() => String(route.params.mangaId));
const onBookshelf = ref(false);
const resumeChapterId = computed(() => {
  const chapterId = manga.value?.readPosition?.chapterId;
  return manga.value?.chapters.some((chapter) => chapter.id === chapterId)
    ? chapterId
    : null;
});

function toChapterItem(chapter: MangaChapter): BookChapterItem<MangaChapter> {
  return {
    id: chapter.id,
    title: chapter.name || `第 ${chapter.chapterNumber} 话`,
    icon: "book-open-variant",
    meta: chapter.pageCount > 0 ? `${chapter.pageCount} 页` : undefined,
    unread: !chapter.isRead,
    data: chapter,
  };
}

async function load(): Promise<void> {
  loading.value = true;
  error.value = "";
  try {
    const detail = await getManga(mangaId.value);
    manga.value = detail;
    onBookshelf.value = await isOnMangaBookshelf(detail.id);
  } catch (value) {
    error.value = getErrorMessage(value, "无法加载漫画详情");
    showError(value, "无法加载漫画详情");
  } finally {
    loading.value = false;
  }
}
onMounted(() => void load());
function read(chapterId: string): void {
  if (!manga.value) return;
  void router.push({
    name: "manga-reader",
    params: { mangaId: manga.value.id, chapterId },
  });
}
function continueReading(): void {
  if (!manga.value) return;
  read(resumeChapterId.value ?? manga.value.chapters[0].id);
}
function goBack(): void {
  if (window.history.state?.back) router.back();
  else void router.replace({ name: "manga" });
}
async function toggleBookshelf(): Promise<void> {
  try {
    if (!manga.value) return;
    if (onBookshelf.value) await removeFromMangaBookshelf(manga.value.id);
    else await addToMangaBookshelf(manga.value.id);
    onBookshelf.value = !onBookshelf.value;
  } catch (errorValue) {
    showError(errorValue, "更新漫画书架失败");
  }
}
</script>
<template>
  <BookDetailView
    :title="manga?.title ?? ''"
    :cover-url="manga?.thumbnailUrl ?? null"
    :author="manga?.author || manga?.artist || '作者未知'"
    :status="manga?.status"
    :tags="manga?.genre ?? []"
    :description="manga?.description"
    :on-bookshelf="onBookshelf"
    :loading="loading"
    :resume-chapter-id="resumeChapterId"
    :can-start-reading="Boolean(manga?.chapters[0])"
    :loaded="Boolean(manga)"
    :error="error"
    error-title="漫画详情加载失败"
    show-inline-loading
    loading-label="正在加载漫画详情"
    back-label="返回漫画"
    @back="goBack"
    @toggle-bookshelf="toggleBookshelf"
    @continue-reading="continueReading"
    @retry="load"
  >
    <template #stats>
      <span><strong>{{ manga?.chapters.length ?? 0 }}</strong> 话</span>
    </template>
    <template #chapters>
      <BookDetailSection v-if="manga" title="章节" :summary="`共 ${manga.chapters.length} 话`">
        <BookChapterList
          :items="manga.chapters.map(toChapterItem)"
          @open="read($event.data.id)"
        />
      </BookDetailSection>
    </template>
  </BookDetailView>
</template>
