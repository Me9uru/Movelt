<script setup lang="ts" generic="T">
import { computed } from "vue";
import BookCollection from "../../layout/BookCollection.vue";
import BookGrid from "../common/BookGrid.vue";
import type { RecommendDiscoveryModel } from "../../types/discovery";
import { useDiscoveryContext } from "../../composables/discovery/useDiscoveryContext";

const props = defineProps<RecommendDiscoveryModel<T>>();
const discoveryContext = useDiscoveryContext();

const collectionItems = computed(() => props.blocks.flatMap((block) => block.items));

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
    empty-message="暂无推荐内容"
    error-title="推荐加载失败"
    grid-class="discovery-grid"
    :has-content="blocks.length > 0"
    :content-while-loading="false"
    @retry="emit('retry')"
  >
    <template #content>
      <section v-for="block in blocks" :key="block.title" class="discovery-block">
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
    </template>
  </BookCollection>
</template>
