<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { NovelSummary, Volume } from "../../domain/content";
import CatalogueBranch from "../../components/novel/CatalogueBranch.vue";
import WorkCover from "../../components/common/WorkCover.vue";
import WorkDescription from "../../components/common/WorkDescription.vue";

const props = defineProps<{
  detail: NovelSummary;
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
const descriptionElement = ref<HTMLElement | null>(null);
const descriptionExpanded = ref(false);
const descriptionCanExpand = ref(false);
let descriptionResizeObserver: ResizeObserver | undefined;
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

function updateDescriptionExpansion(): void {
  const element = descriptionElement.value?.querySelector<HTMLElement>(".work-description");
  if (!element || descriptionExpanded.value) return;
  descriptionCanExpand.value = element.scrollHeight > element.clientHeight + 1;
}

watch(
  () => props.detail.description,
  () => {
    descriptionExpanded.value = false;
    void nextTick(updateDescriptionExpansion);
  },
  { immediate: true },
);

onMounted(() => {
  descriptionResizeObserver = new ResizeObserver(updateDescriptionExpansion);
  if (descriptionElement.value) descriptionResizeObserver.observe(descriptionElement.value);
  void nextTick(updateDescriptionExpansion);
});

onBeforeUnmount(() => descriptionResizeObserver?.disconnect());

function countChapters(volume: Volume): number {
  return volume.chapters.length + volume.sections.reduce((total, section) => total + countChapters(section), 0);
}

</script>

<template>
  <section class="detail-view">
    <section class="book-profile">
      <div class="book-heading">
        <WorkCover
          class="detail-cover"
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
            <var-divider vertical />
            <span><strong>{{ chapterCount }}</strong> 话</span>
            <template v-if="detail.updated_at">
              <var-divider vertical />
              <span>更新于 {{ detail.updated_at }}</span>
            </template>
          </div>
          <div ref="descriptionElement" class="description" :class="{ 'description--expanded': descriptionExpanded }">
            <WorkDescription :content="detail.description" fallback="暂无作品简介。" />
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
          <div v-if="detail.tags.length" class="detail-tags">
            <var-chip v-for="tag in detail.tags" :key="tag" plain>{{ tag }}</var-chip>
          </div>
        </div>

        <div class="detail-rail">
          <div class="detail-actions">
            <var-button :type="onBookshelf ? 'default' : 'primary'" size="large"
              :disabled="loading" @click="emit('toggleBookshelf')">
              <var-icon :name="onBookshelf ? 'check' : 'star'" />
              {{ onBookshelf ? "已加入书架" : "加入书架" }}
            </var-button>
            <var-button v-if="resumeChapterId" size="large" :disabled="loading" @click="emit('continueReading')"><var-icon name="play" />继续阅读</var-button>
          </div>
        </div>
      </div>
    </section>

    <section class="catalogue-section">
      <div class="catalogue-heading">
        <div>
          <h2>作品目录</h2>
        </div>
        <p>共 {{ catalogue.length }} 篇 · {{ chapterCount }} 话</p>
      </div>

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
          <CatalogueBranch v-if="activeVolume === volumeIndex" :volume="volume" :loading="loading"
            @open-chapter="emit('openChapter', $event)" />
        </var-collapse-item>
      </var-collapse>
    </section>
  </section>
</template>
