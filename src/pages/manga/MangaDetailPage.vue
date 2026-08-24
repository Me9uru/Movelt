<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  addToMangaBookshelf,
  getManga,
  isOnMangaBookshelf,
  removeFromMangaBookshelf,
} from "../../services/manga";
import type { MangaDetail } from "../../domain/content";
import { getErrorMessage, showError } from "../../utils/error";
import ErrorState from "../../components/common/ErrorState.vue";
import WorkDescription from "../../components/common/WorkDescription.vue";
import MangaChapterList from "../../components/manga/MangaChapterList.vue";
import WorkCover from "../../components/common/WorkCover.vue";
import LoadingOverlay from "../../components/common/LoadingOverlay.vue";

const route = useRoute();
const router = useRouter();
const manga = ref<MangaDetail | null>(null);
const loading = ref(true);
const error = ref("");
const mangaId = computed(() => String(route.params.mangaId));
const onBookshelf = ref(false);
const descriptionElement = ref<HTMLElement | null>(null);
const descriptionExpanded = ref(false);
const descriptionCanExpand = ref(false);
let descriptionResizeObserver: ResizeObserver | undefined;
const resumeChapterId = computed(() => {
  const chapterId = manga.value?.readPosition?.chapterId;
  return manga.value?.chapters.some((chapter) => chapter.id === chapterId)
    ? chapterId
    : null;
});

function updateDescriptionExpansion(): void {
  const element = descriptionElement.value?.querySelector<HTMLElement>(".work-description");
  if (!element || descriptionExpanded.value) return;
  descriptionCanExpand.value = element.scrollHeight > element.clientHeight + 1;
}

watch(
  () => manga.value?.description,
  () => {
    descriptionExpanded.value = false;
    void nextTick(updateDescriptionExpansion);
  },
);
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
onMounted(() => {
  descriptionResizeObserver = new ResizeObserver(updateDescriptionExpansion);
  if (descriptionElement.value) descriptionResizeObserver.observe(descriptionElement.value);
  void nextTick(updateDescriptionExpansion);
});
onBeforeUnmount(() => descriptionResizeObserver?.disconnect());
function read(chapterId: string): void {
  if (!manga.value) return;
  void router.push({
    name: "manga-reader",
    params: { mangaId: manga.value.id, chapterId },
  });
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
  <section class="manga-detail-view">
    <LoadingOverlay v-if="loading" inline visible label="正在加载漫画详情" />
    <header class="manga-detail-back">
      <var-button text @click="router.push({ name: 'manga' })"><var-icon name="arrow-left" />返回漫画</var-button>
    </header>
    <ErrorState
      v-if="error"
      title="漫画详情加载失败"
      :message="error"
      :loading="loading"
      @retry="load"
    />
    <template v-else-if="manga">
      <article class="manga-profile">
        <WorkCover
          class="manga-detail-cover"
          :cover-url="manga.thumbnailUrl"
          :title="manga.title"
        />
        <div class="manga-detail-copy">
          <h1>{{ manga.title }}</h1>
          <p class="manga-byline">
            {{ manga.author || manga.artist || "作者未知" }} ·
            {{ manga.status }}
          </p>
          <div
            ref="descriptionElement"
            class="manga-description"
            :class="{ 'manga-description--expanded': descriptionExpanded }"
          >
            <WorkDescription :content="manga.description" />
            <button
              v-if="descriptionCanExpand"
              type="button"
              class="description-toggle"
              :aria-expanded="descriptionExpanded"
              @click="descriptionExpanded = !descriptionExpanded"
            >
              {{ descriptionExpanded ? "收起简介" : "展开全部" }}
            </button>
          </div>
          <div class="manga-tags">
            <var-chip v-for="tag in manga.genre" :key="tag" plain>{{
              tag
            }}</var-chip>
          </div>
        </div>
        <div class="manga-detail-rail">
          <div class="manga-detail-actions">
            <var-button
              :type="onBookshelf ? 'default' : 'primary'"
              size="large"
              :disabled="loading"
              @click="toggleBookshelf"
            >
              <var-icon :name="onBookshelf ? 'check' : 'star'" />
              {{ onBookshelf ? "已加入书架" : "加入书架" }}
            </var-button>
            <var-button
              v-if="resumeChapterId"
              size="large"
              :disabled="loading"
              @click="read(resumeChapterId)"
            >
              <var-icon name="play" />
              继续阅读
            </var-button>
            <var-button
              v-else-if="manga.chapters[0]"
              size="large"
              :disabled="loading"
              @click="read(manga.chapters[0].id)"
            >
              <var-icon name="play" />
              开始阅读
            </var-button>
          </div>
        </div>
      </article>
      <section class="manga-chapter-section">
        <header>
          <div>
            <h2>章节</h2>
            <p>{{ manga.chapters.length }} 话</p>
          </div>
        </header>
        <MangaChapterList :chapters="manga.chapters" @open-chapter="read" />
      </section>
    </template>
  </section>
</template>
