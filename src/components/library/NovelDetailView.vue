<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Check, Collection, Star, VideoPlay } from "@element-plus/icons-vue";
import type { ReadingProgress } from "../../domain/library";
import type { NovelDetail, Volume } from "../../services/novel";
import { localEpubAssetUrl, localEpubSourceId } from "../../services/localEpub";
import ReadingProgressBar from "./ReadingProgressBar.vue";
import CatalogueBranch from "./CatalogueBranch.vue";

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

const activeVolume = ref<number | string>("");
const localCoverUrl = ref<string | null>(null);

watch(
  () => [props.detail.id, props.detail.source, props.detail.cover_url] as const,
  async ([bookId, source, coverUrl]) => {
    localCoverUrl.value = source === localEpubSourceId
      ? await localEpubAssetUrl(bookId, coverUrl)
      : coverUrl;
  },
  { immediate: true },
);

const chapterCount = computed(() =>
  props.catalogue.reduce((total, volume) => total + countChapters(volume), 0),
);

watch(
  [() => props.catalogue, () => props.currentProgress?.documentId],
  ([catalogue, currentDocumentId]) => {
    let activeVolumeIndex = catalogue.length > 0 ? 0 : -1;
    if (currentDocumentId) {
      for (const [volumeIndex, volume] of catalogue.entries()) {
        if (containsChapter(volume, currentDocumentId)) {
          activeVolumeIndex = volumeIndex;
          break;
        }
      }
    }

    activeVolume.value = activeVolumeIndex >= 0 ? activeVolumeIndex : "";
  },
  { immediate: true },
);

function countChapters(volume: Volume): number {
  return volume.chapters.length + volume.sections.reduce((total, section) => total + countChapters(section), 0);
}

function containsChapter(volume: Volume, chapterId: string): boolean {
  return volume.chapters.some((chapter) => chapter.id === chapterId)
    || volume.sections.some((section) => containsChapter(section, chapterId));
}
</script>

<template>
  <section class="detail-view">
    <el-card class="book-profile" shadow="never">
      <div class="book-heading">
        <el-image v-if="localCoverUrl" class="detail-cover" :src="localCoverUrl" :alt="detail.title"
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
              <el-tag size="small" round effect="plain">{{ countChapters(volume) }} 话</el-tag>
            </div>
          </template>
          <CatalogueBranch v-if="activeVolume === volumeIndex" :volume="volume" :loading="loading"
            :current-progress="currentProgress" @open-chapter="emit('openChapter', $event)" />
        </el-collapse-item>
      </el-collapse>
    </section>
  </section>
</template>
