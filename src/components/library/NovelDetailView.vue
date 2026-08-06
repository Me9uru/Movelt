<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Check, Collection, Reading, Star, VideoPlay } from "@element-plus/icons-vue";
import type { ReadingProgress } from "../../domain/library";
import type { ChapterSummary, NovelDetail, Volume } from "../../services/novel";
import ReadingProgressBar from "./ReadingProgressBar.vue";

const props = defineProps<{
  detail: NovelDetail;
  catalogue: Volume[];
  loading: boolean;
  onBookshelf: boolean;
  currentProgress: ReadingProgress | null;
}>();

const emit = defineEmits<{
  toggleBookshelf: [];
  continueReading: [];
  openChapter: [chapterId: string];
}>();

const chaptersPerPage = 100;
const activeVolume = ref<number | string>("");
const volumePages = ref<Record<number, number>>({});

const chapterCount = computed(() =>
  props.catalogue.reduce((total, volume) => total + volume.chapters.length, 0),
);

watch(
  [() => props.catalogue, () => props.currentProgress?.documentId],
  ([catalogue, currentDocumentId]) => {
    let activeVolumeIndex = catalogue.length > 0 ? 0 : -1;
    let activeChapterIndex = 0;

    if (currentDocumentId) {
      for (const [volumeIndex, volume] of catalogue.entries()) {
        const chapterIndex = volume.chapters.findIndex((chapter) => chapter.id === currentDocumentId);
        if (chapterIndex >= 0) {
          activeVolumeIndex = volumeIndex;
          activeChapterIndex = chapterIndex;
          break;
        }
      }
    }

    activeVolume.value = activeVolumeIndex >= 0 ? activeVolumeIndex : "";
    volumePages.value = activeVolumeIndex >= 0
      ? { [activeVolumeIndex]: Math.floor(activeChapterIndex / chaptersPerPage) + 1 }
      : {};
  },
  { immediate: true },
);

function volumePage(volumeIndex: number): number {
  return volumePages.value[volumeIndex] ?? 1;
}

function setVolumePage(volumeIndex: number, page: number): void {
  volumePages.value = { ...volumePages.value, [volumeIndex]: page };
}

function visibleChapters(volume: Volume, volumeIndex: number): ChapterSummary[] {
  const start = (volumePage(volumeIndex) - 1) * chaptersPerPage;
  return volume.chapters.slice(start, start + chaptersPerPage);
}

function chapterNumber(volumeIndex: number, chapterIndex: number): string {
  const number = (volumePage(volumeIndex) - 1) * chaptersPerPage + chapterIndex + 1;
  return String(number).padStart(2, "0");
}

function chapterProgressPercent(progress: ReadingProgress): number {
  return Math.round(progress.location * 100);
}
</script>

<template>
  <section class="detail-view">
    <el-card class="book-profile" shadow="never">
      <div class="book-heading">
        <el-image v-if="detail.cover_url" class="detail-cover" :src="detail.cover_url" :alt="detail.title"
          fit="cover" />
        <div v-else class="detail-cover cover-placeholder">
          <el-icon>
            <Collection />
          </el-icon>
        </div>

        <div class="detail-copy">
          <h1>{{ detail.title }}</h1>
          <p class="author">
            <span>{{ detail.author || "佚名" }}</span>
            <template v-if="detail.status">
              <i aria-hidden="true"></i>
              <span>{{ detail.status }}</span>
            </template>
          </p>
          <div class="stats">
            <span><strong>{{ catalogue.length }}</strong> 卷</span>
            <el-divider direction="vertical" />
            <span><strong>{{ chapterCount }}</strong> 章</span>
            <template v-if="detail.updated_at">
              <el-divider direction="vertical" />
              <span>更新于 {{ detail.updated_at }}</span>
            </template>
          </div>
          <p class="description">{{ detail.description || "暂无作品简介。" }}</p>
        </div>

        <div class="detail-rail">
          <ReadingProgressBar v-if="currentProgress" class="detail-progress" :progress="currentProgress" />
          <div class="detail-actions">
            <el-button :type="onBookshelf ? 'default' : 'primary'" :icon="onBookshelf ? Check : Star" size="large"
              :disabled="loading" @click="emit('toggleBookshelf')">
              {{ onBookshelf ? "已加入书架" : "加入书架" }}
            </el-button>
            <el-button v-if="currentProgress" :icon="VideoPlay" size="large" :disabled="loading"
              @click="emit('continueReading')">
              继续阅读 · {{ currentProgress.documentTitle }}
            </el-button>
          </div>
        </div>
      </div>
    </el-card>

    <section class="catalogue-section">
      <div class="catalogue-heading">
        <div>
          <h2>作品目录</h2>
        </div>
        <p>共 {{ catalogue.length }} 卷 · {{ chapterCount }} 章</p>
      </div>

      <el-collapse v-model="activeVolume" accordion class="catalogue">
        <el-collapse-item v-for="(volume, volumeIndex) in catalogue" :key="`${volume.title}-${volumeIndex}`"
          :name="volumeIndex">
          <template #title>
            <div class="volume-title">
              <span class="volume-index">{{ String(volumeIndex + 1).padStart(2, "0") }}</span>
              <strong>{{ volume.title }}</strong>
              <el-tag size="small" round effect="plain">{{ volume.chapters.length }} 章</el-tag>
            </div>
          </template>
          <div v-if="activeVolume === volumeIndex" class="chapter-list">
            <button v-for="(item, chapterIndex) in visibleChapters(volume, volumeIndex)" :key="item.id" type="button"
              :class="{ 'chapter-current': currentProgress?.documentId === item.id }" :disabled="loading"
              @click="emit('openChapter', item.id)">
              <span class="chapter-number">{{ chapterNumber(volumeIndex, chapterIndex) }}</span>
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
          <div v-if="activeVolume === volumeIndex && volume.chapters.length > chaptersPerPage" class="chapter-pagination">
            <span>
              第 {{ (volumePage(volumeIndex) - 1) * chaptersPerPage + 1 }}–{{
                Math.min(volumePage(volumeIndex) * chaptersPerPage, volume.chapters.length)
              }} 章
            </span>
            <el-pagination small background layout="prev, pager, next" :pager-count="5" :page-size="chaptersPerPage"
              :total="volume.chapters.length" :current-page="volumePage(volumeIndex)"
              @update:current-page="setVolumePage(volumeIndex, $event)" />
          </div>
        </el-collapse-item>
      </el-collapse>
    </section>
  </section>
</template>
