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
import BookChapterList from "../../components/detail/BookChapterList.vue";
import type { BookChapterItem } from "../../types/book";
import BookDetailLayout from "../../layout/BookDetailLayout.vue";
import ErrorState from "../../components/common/ErrorState.vue";
import LoadingOverlay from "../../components/common/LoadingOverlay.vue";

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

function read(chapterId: string): void {
  if (!manga.value) return;
  void router.push({ name: "manga-reader", params: { mangaId: manga.value.id, chapterId } });
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
function continueReading(): void {
  if (!manga.value) return;
  read(resumeChapterId.value ?? manga.value.chapters[0].id);
}
function goBack(): void {
  if (window.history.state?.back) router.back();
  else void router.replace({ name: "manga" });
}
async function toggleBookshelf(): Promise<void> {
  if (!manga.value) return;
  try {
    if (onBookshelf.value) await removeFromMangaBookshelf(manga.value.id);
    else await addToMangaBookshelf(manga.value.id);
    onBookshelf.value = !onBookshelf.value;
  } catch (value) {
    showError(value, "更新漫画书架失败");
  }
}
</script>
<template>
  <div>
    <header class="topbar book-detail-topbar">
      <div class="topbar-inner detail-topbar">
        <var-button text @click="goBack"><var-icon name="arrow-left" />返回漫画</var-button>
      </div>
    </header>
    <section class="detail-view">
      <LoadingOverlay v-if="loading" inline visible label="正在加载漫画详情" />
      <ErrorState v-if="error" title="漫画详情加载失败" :message="error" :loading="loading" @retry="load" />
  <BookDetailLayout
    v-else-if="manga"
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
    :stats="[{ value: manga.chapters.length, label: '话' }]"
    section-title="章节"
    :section-summary="`共 ${manga.chapters.length} 话`"
    @toggle-bookshelf="toggleBookshelf"
    @continue-reading="continueReading"
  >
    <BookChapterList
      :items="manga.chapters.map(toChapterItem)"
      @open="read($event.data.id)"
    />
  </BookDetailLayout>
    </section>
  </div>
</template>
