<script setup lang="ts">
import BookDetailLayout from "./BookDetailLayout.vue";
import ErrorState from "../common/ErrorState.vue";
import LoadingOverlay from "../common/LoadingOverlay.vue";

withDefaults(defineProps<{
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
  loaded?: boolean;
  error?: string;
  errorTitle?: string;
  showInlineLoading?: boolean;
  loadingLabel?: string;
  backLabel?: string;
}>(), {
  status: undefined,
  descriptionFallback: undefined,
  resumeChapterId: null,
  canStartReading: false,
  loaded: true,
  error: "",
  errorTitle: "加载失败",
  showInlineLoading: false,
  loadingLabel: "正在加载详情",
  backLabel: "",
});

const emit = defineEmits<{
  toggleBookshelf: [];
  continueReading: [];
  retry: [];
  back: [];
}>();
</script>

<template>
  <div>
    <header v-if="backLabel" class="topbar book-detail-topbar">
      <div class="topbar-inner detail-topbar">
        <var-button text @click="emit('back')">
          <var-icon name="arrow-left" />
          {{ backLabel }}
        </var-button>
      </div>
    </header>
    <section class="detail-view">
      <LoadingOverlay v-if="showInlineLoading && loading" inline visible :label="loadingLabel" />
      <ErrorState v-if="error" :title="errorTitle" :message="error" :loading="loading" @retry="emit('retry')" />
      <BookDetailLayout
        v-else-if="loaded"
        :title="title"
        :cover-url="coverUrl"
        :author="author"
        :status="status"
        :tags="tags"
        :description="description"
        :description-fallback="descriptionFallback"
        :on-bookshelf="onBookshelf"
        :loading="loading"
        :resume-chapter-id="resumeChapterId"
        :can-start-reading="canStartReading"
        @toggle-bookshelf="emit('toggleBookshelf')"
        @continue-reading="emit('continueReading')"
      >
        <template #stats><slot name="stats" /></template>
        <template #chapters><slot name="chapters" /></template>
      </BookDetailLayout>
    </section>
  </div>
</template>
