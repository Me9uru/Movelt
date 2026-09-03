<script setup lang="ts" generic="T">
import BookCollection from "../../layout/BookCollection.vue";
import type { RankingDiscoveryModel } from "../../types/discovery";
import { useDiscoveryContext } from "../../composables/discovery/useDiscoveryContext";
import AppTabs from "../common/AppTabs.vue";

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
    :empty-state="{
      icon: 'bookmark',
      title: `这个榜单还没有${discoveryContext.subjectLabel}`,
    }"
    error-title="榜单加载失败"
    grid-class="discovery-grid"
    :content-while-loading="false"
    @open="emit('open', $event.data)"
    @retry="emit('retry')"
  >
    <template #header>
      <AppTabs
        v-if="periods?.length"
        class="ranking-period-tabs"
        variant="secondary"
        :model-value="String(days)"
        :tabs="periods.map((period) => ({
          name: String(period.value),
          label: period.label,
        }))"
        accessibility-label="榜单时间范围"
        @update:model-value="emit('update:days', Number($event))"
      />
    </template>
  </BookCollection>
</template>
