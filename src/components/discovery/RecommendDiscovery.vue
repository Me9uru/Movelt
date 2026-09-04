<script setup lang="ts" generic="T">
import { computed } from "vue";
import BookCollection from "../../layout/BookCollection.vue";
import BookGrid from "../book/BookGrid.vue";
import type { RecommendDiscoveryModel } from "../../types/discovery";
import { useDiscoveryContext } from "../../composables/discovery/useDiscoveryContext";
import { useProgressiveRender } from "../../composables/useProgressiveRender";

const props = defineProps<RecommendDiscoveryModel<T>>();
const discoveryContext = useDiscoveryContext();

const collectionItems = computed(() => props.blocks.flatMap((block) => block.items));
const progressiveKeys = computed(() =>
  props.blocks.flatMap((block) =>
    block.items.map((item) => `${block.title}:${item.id}`),
  ),
);
const { visibleCount, setSentinel, hasMore } = useProgressiveRender(
  () => progressiveKeys.value,
);
const visibleBlocks = computed(() => {
  let remaining = visibleCount.value;
  return props.blocks.flatMap((block) => {
    if (remaining <= 0) return [];
    const items = block.items.slice(0, remaining);
    remaining -= items.length;
    return items.length > 0 ? [{ ...block, items }] : [];
  });
});

const emit = defineEmits<{
  open: [item: T];
  retry: [];
}>();
</script>

<template>
  <BookCollection
    :items="collectionItems"
    :loading="loading"
    :error="error"
    :empty-state="{ icon: 'bookmark', title: '暂无推荐内容' }"
    error-title="推荐加载失败"
    grid-class="discovery-grid"
    :has-content="blocks.length > 0"
    :content-while-loading="false"
    @retry="emit('retry')"
  >
    <template #content>
      <section
        v-for="block in visibleBlocks"
        :key="block.title"
        class="discovery-block"
      >
        <div class="section-heading">
          <h2>{{ block.title }}</h2>
          <var-chip class="count-tag" plain>
            {{ block.items.length }} {{ discoveryContext.countUnit }}
          </var-chip>
        </div>
        <BookGrid
          class="discovery-grid"
          :items="block.items"
          @open="emit('open', $event.data)"
        />
      </section>
      <div
        v-if="hasMore"
        :ref="setSentinel"
        class="book-collection__load-more"
        aria-hidden="true"
      />
    </template>
  </BookCollection>
</template>
