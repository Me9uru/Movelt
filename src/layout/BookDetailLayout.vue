<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  useId,
  watch,
} from "vue";
import AppEmptyState from "../components/common/AppEmptyState.vue";
import BookCover from "../components/book/BookCover.vue";

interface BookDetailLayoutProps {
  title: string;
  coverUrl: string | null;
  author: string;
  status?: string | null;
  tags: string[];
  description: string | null | undefined;
  descriptionFallback?: string;
  onBookshelf: boolean;
  loading: boolean;
  resumeChapterId?: string | null;
  canStartReading?: boolean;
  hasChapters?: boolean;
  stats?: { value?: string | number; label: string }[];
  sectionTitle: string;
  sectionSummary: string;
}

const props = withDefaults(
  defineProps<BookDetailLayoutProps>(),
  {
    descriptionFallback: "暂无简介。",
    canStartReading: false,
    hasChapters: true,
    stats: () => [],
  },
);

const emit = defineEmits<{
  toggleBookshelf: [];
  continueReading: [];
}>();

const descriptionId = useId();
const catalogueId = useId();
const hasStats = computed(() => props.stats.length > 0);

const descriptionElement = ref<HTMLElement | null>(null);
const descriptionExpanded = ref(false);
const descriptionCanExpand = ref(false);
let descriptionResizeObserver: ResizeObserver | undefined;

const updateDescriptionExpansion = (): void => {
  const element =
    descriptionElement.value?.querySelector<HTMLElement>(".book-description");
  if (!element || descriptionExpanded.value) return;
  descriptionCanExpand.value = element.scrollHeight > element.clientHeight + 1;
}

watch(
  () => props.description,
  () => {
    descriptionExpanded.value = false;
    void nextTick(updateDescriptionExpansion);
  },
  { immediate: true },
);

onMounted(() => {
  descriptionResizeObserver = new ResizeObserver(updateDescriptionExpansion);
  if (descriptionElement.value)
    descriptionResizeObserver.observe(descriptionElement.value);
  void nextTick(updateDescriptionExpansion);
});

onBeforeUnmount(() => descriptionResizeObserver?.disconnect());
</script>

<template>
  <section class="book-detail-view">
    <article class="book-detail-profile">
      <BookCover
        class="book-detail-cover"
        :title="title"
        :cover-url="coverUrl"
      />
      <div class="book-detail-copy">
        <h1>{{ title }}</h1>
        <p class="book-detail-byline">
          <span>{{ author }}</span>
          <template v-if="status">
            <i aria-hidden="true"></i>
            <span>{{ status }}</span>
          </template>
        </p>
        <div v-if="hasStats" class="book-detail-stats">
          <template
            v-for="(stat, index) in stats"
            :key="`${stat.value}-${stat.label}`"
          >
            <var-divider v-if="index" vertical />
            <span
              ><strong v-if="stat.value !== undefined">{{ stat.value }}</strong>
              {{ stat.label }}</span
            >
          </template>
        </div>
      </div>
      <div class="book-detail-description-block">
        <var-card
          variant="filled"
          :elevation="false"
          class="book-detail-description-card"
          :class="{
            'book-detail-description-card--expanded': descriptionExpanded,
            'book-detail-description-card--no-stats': !hasStats,
          }"
          :aria-labelledby="`${descriptionId}-title`"
        >
          <template #title="{ slotClass }">
            <h2
              :id="`${descriptionId}-title`"
              :class="[slotClass, 'book-detail-description-title']"
            >
              简介
            </h2>
          </template>
          <div ref="descriptionElement" class="book-detail-description">
            <div
              :id="descriptionId"
              class="book-description"
              v-html="description || descriptionFallback"
            />
            <var-button
              v-if="descriptionCanExpand"
              text
              class="description-toggle"
              :aria-expanded="descriptionExpanded"
              :aria-controls="descriptionId"
              @click="descriptionExpanded = !descriptionExpanded"
            >
              {{ descriptionExpanded ? "收起简介" : "展开全部" }}
            </var-button>
          </div>
        </var-card>
        <div v-if="tags.length" class="book-detail-tags" aria-label="作品标签">
          <span v-for="tag in tags" :key="tag" class="book-detail-tag">
            {{ tag }}
          </span>
        </div>
      </div>

      <aside class="book-detail-rail">
        <div class="book-detail-actions">
          <var-button
            v-if="resumeChapterId || canStartReading"
            type="primary"
            size="large"
            :elevation="false"
            :disabled="loading"
            @click="emit('continueReading')"
          >
            <var-icon name="play-circle-outline" aria-hidden="true" />
            {{ resumeChapterId ? "继续阅读" : "开始阅读" }}
          </var-button>
          <var-button
            type="primary"
            size="large"
            :tonal="onBookshelf"
            :outline="!onBookshelf"
            :text="!onBookshelf"
            :elevation="false"
            :disabled="loading"
            :aria-pressed="onBookshelf"
            :aria-label="onBookshelf ? '从书架移除' : '加入书架'"
            @click="emit('toggleBookshelf')"
          >
            <var-icon :name="onBookshelf ? 'check' : 'bookmark-outline'" aria-hidden="true" />
            {{ onBookshelf ? "已加入书架" : "加入书架" }}
          </var-button>
        </div>
      </aside>
    </article>
    <section class="catalogue-section" :aria-labelledby="catalogueId">
      <div class="catalogue-heading">
        <h2 :id="catalogueId">{{ sectionTitle }}</h2>
        <p>{{ sectionSummary }}</p>
      </div>
      <slot v-if="hasChapters" />
      <AppEmptyState v-else icon="notebook" title="暂无章节" description="作品更新后，章节将在这里显示。" />
    </section>
  </section>
</template>
