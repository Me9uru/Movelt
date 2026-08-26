<script setup lang="ts" generic="T">
import { computed } from "vue";
import BookCollection from "../../layout/BookCollection.vue";
import BookGrid from "../common/BookGrid.vue";
import type { BookGridItem } from "../../types/book";
import type { RecommendDiscoveryProps } from "../../types/discovery";

const props = defineProps<RecommendDiscoveryProps<T>>();

const collectionItems = computed(() => props.blocks.flatMap((block) => block.items));

const emit = defineEmits<{
  open: [item: BookGridItem<T>];
  retry: [];
}>();
</script>

<template>
  <BookCollection
    :items="collectionItems"
    :loading="loading"
    :error="error"
    :empty-message="emptyMessage"
    :error-title="errorTitle"
    grid-class="discovery-grid"
    :has-content="blocks.length > 0"
    :content-while-loading="false"
    @retry="emit('retry')"
  >
    <template #content>
      <section v-for="block in blocks" :key="block.title" class="discovery-block">
        <div class="section-heading">
          <h2>{{ block.title }}</h2>
          <var-chip class="count-tag" plain>{{ block.items.length }} 本</var-chip>
        </div>
        <BookGrid
          class="discovery-grid"
          :items="block.items"
          @open="emit('open', $event)"
        />
      </section>
    </template>
  </BookCollection>
</template>
