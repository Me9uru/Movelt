<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Check, Star, VideoPlay } from "@element-plus/icons-vue";
import type { NovelDetail, Volume } from "../../services/novel";
import CatalogueBranch from "../../components/library/CatalogueBranch.vue";
import NovelCover from "../../components/library/NovelCover.vue";

const props = defineProps<{
  detail: NovelDetail;
  catalogue: Volume[];
  loading: boolean;
  onBookshelf: boolean;
  resumeChapterId?: string | null;
}>();

const emit = defineEmits<{
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

</script>

<template>
  <section class="detail-view">
    <el-card class="book-profile" shadow="never">
      <div class="book-heading">
        <NovelCover
          class="detail-cover"
          :source="detail.source"
          :novel-id="detail.id"
          :title="detail.title"
          :cover-url="detail.cover_url"
        />

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
            <span><strong>{{ catalogue.length }}</strong> 篇</span>
            <el-divider direction="vertical" />
            <span><strong>{{ chapterCount }}</strong> 话</span>
            <template v-if="detail.updated_at">
              <el-divider direction="vertical" />
              <span>更新于 {{ detail.updated_at }}</span>
            </template>
          </div>
          <p class="description">{{ detail.description || "暂无作品简介。" }}</p>
          <div v-if="detail.tags.length" class="detail-tags">
            <el-tag v-for="tag in detail.tags" :key="tag" effect="plain">{{ tag }}</el-tag>
          </div>
        </div>

        <div class="detail-rail">
          <div class="detail-actions">
            <el-button :type="onBookshelf ? 'default' : 'primary'" :icon="onBookshelf ? Check : Star" size="large"
              :disabled="loading" @click="emit('toggleBookshelf')">
              {{ onBookshelf ? "已加入书架" : "加入书架" }}
            </el-button>
            <el-button v-if="resumeChapterId" :icon="VideoPlay" size="large" :disabled="loading" @click="emit('continueReading')">继续阅读</el-button>
          </div>
        </div>
      </div>
    </el-card>

    <section class="catalogue-section">
      <div class="catalogue-heading">
        <div>
          <h2>作品目录</h2>
        </div>
        <p>共 {{ catalogue.length }} 篇 · {{ chapterCount }} 话</p>
      </div>

      <el-collapse v-model="activeVolume" accordion class="catalogue">
        <el-collapse-item v-for="(volume, volumeIndex) in catalogue" :key="`${volume.title}-${volumeIndex}`"
          class="catalogue-volume" :name="volumeIndex">
          <template #title>
            <div class="volume-title volume-title--part">
              <span class="volume-index">{{ String(volumeIndex + 1).padStart(2, "0") }}</span>
              <strong>{{ volume.title }}</strong>
              <el-tag class="count-tag" size="small" effect="plain">{{ countChapters(volume) }} 话</el-tag>
            </div>
          </template>
          <CatalogueBranch v-if="activeVolume === volumeIndex" :volume="volume" :loading="loading"
            @open-chapter="emit('openChapter', $event)" />
        </el-collapse-item>
      </el-collapse>
    </section>
  </section>
</template>
