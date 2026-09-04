<script setup lang="ts" generic="T">
import {
  computed,
  onBeforeUnmount,
  onMounted,
  ref,
  useSlots,
  watch,
} from "vue";
import BookGrid from "../components/book/BookGrid.vue";
import AppEmptyState from "../components/common/AppEmptyState.vue";
import ErrorState from "../components/common/ErrorState.vue";
import LoadingOverlay from "../components/common/LoadingOverlay.vue";
import { useProgressiveRender } from "../composables/useProgressiveRender";
import type { BookCollectionState, BookGridItem } from "../types/book";

interface BookCollectionProps<T> {
  items: BookGridItem<T>[] | null;
  loading: boolean;
  error?: string;
  pagination?: { page: number; last: number } | null;
  disabled?: boolean;
  promptState?: BookCollectionState;
  emptyState?: BookCollectionState;
  errorTitle?: string;
  gridClass?: string;
  hasContent?: boolean;
  contentWhileLoading?: boolean;
}

// Vue 会将缺省的 Boolean prop 转成 false。这里显式保留 undefined，才能让
// hasContent 未传入时回退到 items 的实际长度。
const props = withDefaults(defineProps<BookCollectionProps<T>>(), {
  hasContent: undefined,
});
const slots = useSlots();

const defaultEmptyState: BookCollectionState = {
  icon: "bookmark",
  title: "暂无作品",
};
const defaultPromptState: BookCollectionState = {
  icon: "magnify",
  title: "暂无可展示的作品",
};

const emit = defineEmits<{
  open: [item: BookGridItem<T>];
  retry: [];
  loadMore: [page: number];
}>();

const loadMoreSentinel = ref<HTMLElement | null>(null);
let loadMoreObserver: IntersectionObserver | undefined;

const hasMore = computed(() =>
  Boolean(props.pagination && props.pagination.page < props.pagination.last),
);
const hasContent = computed(
  () => props.hasContent ?? Boolean(props.items?.length),
);
const shouldShowContent = computed(
  () => hasContent.value && ((props.contentWhileLoading ?? true) || !props.loading),
);
const progressiveKeys = computed(
  () => props.items?.map((item) => item.id) ?? [],
);
const progressiveEnabled = (): boolean => !slots.content;
const {
  visibleCount,
  setSentinel: setProgressiveSentinel,
  hasMore: hasMoreToRender,
} = useProgressiveRender(
  () => progressiveKeys.value,
  progressiveEnabled,
);
const renderedItems = computed(() =>
  props.items?.slice(0, visibleCount.value) ?? null,
);

onMounted(() => {
  loadMoreObserver = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (
          entry.isIntersecting &&
          hasMore.value &&
          !props.loading &&
          props.pagination
        ) {
          emit("loadMore", props.pagination.page + 1);
        }
      }
    },
    { rootMargin: "400px 0px" },
  );
});

watch(loadMoreSentinel, (element) => {
  loadMoreObserver?.disconnect();
  if (element) loadMoreObserver?.observe(element);
});

onBeforeUnmount(() => loadMoreObserver?.disconnect());
</script>

<template>
  <section class="book-collection">
    <slot name="header" />
    <ErrorState
      v-if="error"
      :title="errorTitle ?? '加载失败'"
      :message="error"
      :loading="loading"
      @retry="emit('retry')"
    />
    <slot v-else-if="shouldShowContent" name="content" :items="items">
      <BookGrid
        :class="gridClass"
        :items="renderedItems!"
        :disabled="disabled || loading"
        @open="emit('open', $event)"
      />
    </slot>
    <slot v-else-if="loading" name="loading">
      <LoadingOverlay inline visible label="正在加载内容" />
    </slot>
    <slot v-else-if="items" name="empty">
      <AppEmptyState v-bind="emptyState ?? defaultEmptyState" />
    </slot>
    <slot v-else name="prompt">
      <AppEmptyState v-bind="promptState ?? defaultPromptState" />
    </slot>
    <div
      v-if="shouldShowContent && hasMoreToRender"
      :ref="setProgressiveSentinel"
      class="book-collection__load-more"
      aria-hidden="true"
    />
    <div
      v-if="items?.length && hasMore && !hasMoreToRender"
      ref="loadMoreSentinel"
      class="book-collection__load-more"
    >
      <span v-if="loading">加载中…</span>
    </div>
  </section>
</template>
