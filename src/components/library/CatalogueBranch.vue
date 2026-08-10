<script setup lang="ts">
import { computed, ref } from "vue";
import { Reading } from "@element-plus/icons-vue";
import type { ReadingProgress } from "../../domain/library";
import type { ChapterSummary, Volume } from "../../services/novel";

const props = defineProps<{
  volume: Volume;
  loading: boolean;
  currentProgress: ReadingProgress | null;
  depth?: number;
}>();

const emit = defineEmits<{
  openChapter: [chapterId: string];
}>();

const chaptersPerPage = 100;
const page = ref(1);
const activeSections = ref<number[]>([0]);

const visibleChapters = computed<ChapterSummary[]>(() => {
  const start = (page.value - 1) * chaptersPerPage;
  return props.volume.chapters.slice(start, start + chaptersPerPage);
});

function countChapters(volume: Volume): number {
  return volume.chapters.length + volume.sections.reduce((total, section) => total + countChapters(section), 0);
}

function chapterProgressPercent(progress: ReadingProgress): number {
  return Math.round(progress.location * 100);
}
</script>

<template>
  <el-collapse v-if="volume.sections.length" v-model="activeSections" class="catalogue catalogue-nested">
    <el-collapse-item v-for="(section, index) in volume.sections" :key="`${section.title}-${index}`"
      class="catalogue-section-item" :name="index">
      <template #title>
        <div class="volume-title volume-title--section">
          <span class="volume-index">{{ String(index + 1).padStart(2, "0") }}</span>
          <strong>{{ section.title }}</strong>
          <el-tag size="small" effect="plain">{{ countChapters(section) }} 话</el-tag>
        </div>
      </template>
      <CatalogueBranch :volume="section" :loading="loading" :current-progress="currentProgress" :depth="(depth ?? 0) + 1"
        @open-chapter="emit('openChapter', $event)" />
    </el-collapse-item>
  </el-collapse>

  <div v-if="volume.chapters.length" class="chapter-list">
    <button v-for="(item, index) in visibleChapters" :key="item.id" type="button"
      :class="{ 'chapter-current': currentProgress?.documentId === item.id }" :disabled="loading"
      @click="emit('openChapter', item.id)">
      <span class="chapter-number">{{ String((page - 1) * chaptersPerPage + index + 1).padStart(2, "0") }}</span>
      <span>
        {{ item.title }}
        <small v-if="currentProgress?.documentId === item.id">
          上次读到 {{ chapterProgressPercent(currentProgress) }}%
        </small>
      </span>
      <el-icon v-if="currentProgress?.documentId === item.id">
        <Reading />
      </el-icon>
    </button>
  </div>
  <div v-if="volume.chapters.length > chaptersPerPage" class="chapter-pagination">
    <span>第 {{ (page - 1) * chaptersPerPage + 1 }}–{{ Math.min(page * chaptersPerPage, volume.chapters.length) }} 话</span>
    <el-pagination small background layout="prev, pager, next" :pager-count="5" :page-size="chaptersPerPage"
      :total="volume.chapters.length" :current-page="page" @update:current-page="page = $event" />
  </div>
</template>
