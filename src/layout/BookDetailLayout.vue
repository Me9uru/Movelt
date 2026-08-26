<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from "vue";
import BookCover from "../components/common/BookCover.vue";
import type { BookDetailLayoutProps } from "../types/book";

const props = withDefaults(
  defineProps<BookDetailLayoutProps>(),
  {
    descriptionFallback: "暂无简介。",
    canStartReading: false,
    stats: () => [],
  },
);

const emit = defineEmits<{
  toggleBookshelf: [];
  continueReading: [];
}>();

const hasStats = computed(() => props.stats.length > 0);

const descriptionElement = ref<HTMLElement | null>(null);
const descriptionExpanded = ref(false);
const descriptionCanExpand = ref(false);
let descriptionResizeObserver: ResizeObserver | undefined;

function updateDescriptionExpansion(): void {
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
        <div
          ref="descriptionElement"
          class="book-detail-description"
          :class="{
            'book-detail-description--expanded': descriptionExpanded,
            'book-detail-description--no-stats': !hasStats,
          }"
        >
          <div
            class="book-description"
            v-html="description || descriptionFallback"
          />
          <var-button
            v-if="descriptionCanExpand"
            text
            class="description-toggle"
            :aria-expanded="descriptionExpanded"
            @click="descriptionExpanded = !descriptionExpanded"
          >
            {{ descriptionExpanded ? "收起简介" : "展开全部" }}
          </var-button>
        </div>
        <div v-if="tags.length" class="book-detail-tags">
          <var-chip v-for="tag in tags" :key="tag" plain>{{ tag }}</var-chip>
        </div>
      </div>

      <aside class="book-detail-rail">
        <div class="book-detail-actions">
          <var-button
            :type="onBookshelf ? 'default' : 'primary'"
            size="large"
            :disabled="loading"
            @click="emit('toggleBookshelf')"
          >
            <var-icon :name="onBookshelf ? 'check' : 'star'" />
            {{ onBookshelf ? "已加入书架" : "加入书架" }}
          </var-button>
          <var-button
            v-if="resumeChapterId"
            size="large"
            :disabled="loading"
            @click="emit('continueReading')"
          >
            <var-icon name="play" />继续阅读
          </var-button>
          <var-button
            v-else-if="canStartReading"
            size="large"
            :disabled="loading"
            @click="emit('continueReading')"
          >
            <var-icon name="play" />开始阅读
          </var-button>
        </div>
      </aside>
    </article>
    <section class="catalogue-section">
      <div class="catalogue-heading">
        <h2>{{ sectionTitle }}</h2>
        <p>{{ sectionSummary }}</p>
      </div>
      <slot />
    </section>
  </section>
</template>
