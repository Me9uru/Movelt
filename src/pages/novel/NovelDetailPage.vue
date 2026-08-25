<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { ChapterSummary, NovelSummary, Volume } from "../../domain/novel";
import BookChapterList from "../../components/book/BookChapterList.vue";
import type {
  BookChapterGroup,
  BookChapterItem,
} from "../../types/book";
import BookDetailView from "../../components/book/BookDetailView.vue";
import BookDetailSection from "../../components/book/BookDetailSection.vue";

const props = defineProps<{
  detail: NovelSummary;
  catalogue: Volume[];
  loading: boolean;
  onBookshelf: boolean;
  resumeChapterId?: string | null;
  backLabel: string;
}>();

const emit = defineEmits<{
  back: [];
  toggleBookshelf: [];
  continueReading: [];
  openChapter: [chapterId: string];
}>();

const activeVolume = ref<number | string>("");
const chapterCount = computed(() =>
  props.catalogue.reduce((total, volume) => total + countChapters(volume), 0),
);

watch(
  [() => props.catalogue],
  ([catalogue]) => {
    let activeVolumeIndex = catalogue.length > 0 ? 0 : -1;
    activeVolume.value = activeVolumeIndex >= 0 ? activeVolumeIndex : "";
  },
  { immediate: true },
);

function countChapters(volume: Volume): number {
  return volume.chapters.length + volume.sections.reduce((total, section) => total + countChapters(section), 0);
}

const chaptersPerPage = 100;

function toChapterItem(chapter: ChapterSummary): BookChapterItem<ChapterSummary> {
  return { id: chapter.id, title: chapter.title, data: chapter };
}

function toGroup(volume: Volume): BookChapterGroup<ChapterSummary> {
  return {
    title: volume.title,
    count: countChapters(volume),
    chapters: volume.chapters.map(toChapterItem),
    groups: volume.sections.map(toGroup),
  };
}
</script>

<template>
  <BookDetailView
    :title="detail.title"
    :cover-url="detail.cover_url"
    :author="detail.author || '佚名'"
    :status="detail.status"
    :tags="detail.tags"
    :description="detail.description"
    description-fallback="暂无作品简介。"
    :on-bookshelf="onBookshelf"
    :loading="loading"
    :resume-chapter-id="resumeChapterId"
    :back-label="backLabel"
    @back="emit('back')"
    @toggle-bookshelf="emit('toggleBookshelf')"
    @continue-reading="emit('continueReading')"
  >
    <template #stats>
      <span><strong>{{ catalogue.length }}</strong> 篇</span>
      <var-divider vertical />
      <span><strong>{{ chapterCount }}</strong> 话</span>
      <template v-if="detail.updated_at">
        <var-divider vertical />
        <span>更新于 {{ detail.updated_at }}</span>
      </template>
    </template>
    <template #chapters>
      <BookDetailSection title="作品目录" :summary="`共 ${catalogue.length} 篇 · ${chapterCount} 话`">

        <var-collapse v-model="activeVolume" accordion class="catalogue">
          <var-collapse-item v-for="(volume, volumeIndex) in catalogue" :key="`${volume.title}-${volumeIndex}`"
            class="catalogue-volume" :name="volumeIndex">
            <template #title>
              <div class="volume-title volume-title--part">
                <span class="volume-index">{{ String(volumeIndex + 1).padStart(2, "0") }}</span>
                <strong>{{ volume.title }}</strong>
                <var-chip class="count-tag" size="small" plain>{{ countChapters(volume) }} 话</var-chip>
              </div>
            </template>
            <BookChapterList v-if="activeVolume === volumeIndex"
              :groups="volume.sections.map(toGroup)"
              :items="volume.chapters.map(toChapterItem)"
              :loading="loading"
              :page-size="chaptersPerPage"
              @open="emit('openChapter', $event.data.id)" />
          </var-collapse-item>
        </var-collapse>
      </BookDetailSection>
    </template>
  </BookDetailView>
</template>
