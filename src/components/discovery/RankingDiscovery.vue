<script setup lang="ts" generic="T">
import BookCollection from "../../layout/BookCollection.vue";
import type { BookGridItem } from "../../types/book";
import type { RankingDiscoveryProps } from "../../types/discovery";

defineProps<RankingDiscoveryProps<T>>();

const emit = defineEmits<{
  open: [item: BookGridItem<T>];
  retry: [];
  "update:days": [value: number];
}>();
</script>

<template>
  <BookCollection
    :items="items"
    :loading="loading"
    :error="error"
    :empty-message="emptyMessage"
    :error-title="errorTitle"
    grid-class="discovery-grid"
    :content-while-loading="false"
    @open="emit('open', $event)"
    @retry="emit('retry')"
  >
    <template #header>
      <var-tabs
        v-if="periods?.length"
        class="ranking-period-tabs"
        :active="String(days)"
        aria-label="榜单时间范围"
        @update:active="emit('update:days', Number($event))"
      >
        <var-tab
          v-for="period in periods"
          :key="period.value"
          :name="String(period.value)"
        >
          {{ period.label }}
        </var-tab>
      </var-tabs>
    </template>
  </BookCollection>
</template>
