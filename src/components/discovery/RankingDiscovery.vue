<script setup lang="ts" generic="T">
import BookCollection from "../../layout/BookCollection.vue";
import type { RankingDiscoveryModel } from "../../types/discovery";
import { useDiscoveryContext } from "../../composables/discovery/useDiscoveryContext";

defineProps<RankingDiscoveryModel<T>>();
const discoveryContext = useDiscoveryContext();

const emit = defineEmits<{
  open: [item: T];
  retry: [];
  "update:days": [value: number];
}>();
</script>

<template>
  <BookCollection
    :items="items"
    :loading="loading"
    :error="error"
    :empty-message="`这个榜单还没有${discoveryContext.subjectLabel}`"
    error-title="榜单加载失败"
    grid-class="discovery-grid"
    :content-while-loading="false"
    @open="emit('open', $event.data)"
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
