<script setup lang="ts" generic="T">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import BookGrid from "../components/common/BookGrid.vue";
import ErrorState from "../components/common/ErrorState.vue";
import type { BookCollectionProps, BookGridItem } from "../types/book";

// Vue 会将缺省的 Boolean prop 转成 false。这里显式保留 undefined，才能让
// hasContent 未传入时回退到 items 的实际长度。
const props = withDefaults(defineProps<BookCollectionProps<T>>(), {
  hasContent: undefined,
});

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
        :items="items!"
        :disabled="disabled || loading"
        @open="emit('open', $event)"
      />
    </slot>
    <slot v-else-if="loading" name="loading">
      <BookGrid :class="gridClass" :items="[]" loading />
    </slot>
    <slot v-else-if="items" name="empty">
      <var-result type="empty" :description="emptyMessage ?? '暂无作品'" />
    </slot>
    <slot v-else name="prompt">
      <var-result
        type="empty"
        :description="promptMessage ?? '暂无可展示的作品'"
      />
    </slot>
    <div
      v-if="items?.length && hasMore"
      ref="loadMoreSentinel"
      class="book-collection__load-more"
    >
      <span v-if="loading">加载中…</span>
    </div>
  </section>
</template>
