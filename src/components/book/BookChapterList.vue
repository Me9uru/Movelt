<script setup lang="ts" generic="T">
import { computed, ref } from "vue";
import type { BookChapterGroup, BookChapterItem } from "../../types/book";

const props = withDefaults(
  defineProps<{
    items?: BookChapterItem<T>[];
    groups?: BookChapterGroup<T>[];
    loading?: boolean;
    pageSize?: number;
  }>(),
  { loading: false },
);

const emit = defineEmits<{ open: [item: BookChapterItem<T>] }>();

const numbered = computed(() => props.pageSize != null);
const page = ref(1);
const activeSections = ref<number[]>([0]);

const visibleItems = computed<BookChapterItem<T>[]>(() => {
  if (!props.items) return [];
  if (!props.pageSize) return props.items;
  const start = (page.value - 1) * props.pageSize;
  return props.items.slice(start, start + props.pageSize);
});

function rowNumber(index: number): string {
  return String((page.value - 1) * (props.pageSize ?? 1) + index + 1).padStart(2, "0");
}
</script>

<template>
  <var-collapse v-if="groups?.length" v-model="activeSections" class="catalogue catalogue-nested">
    <var-collapse-item
      v-for="(group, index) in groups"
      :key="`${group.title}-${index}`"
      class="catalogue-section-item"
      :name="index"
    >
      <template #title>
        <div class="volume-title volume-title--section">
          <span class="volume-index">{{ String(index + 1).padStart(2, "0") }}</span>
          <strong>{{ group.title }}</strong>
          <var-chip class="count-tag" size="small">{{ group.count }} 话</var-chip>
        </div>
      </template>
      <BookChapterList
        :groups="group.groups"
        :items="group.chapters"
        :loading="loading"
        :page-size="pageSize"
        @open="emit('open', $event)"
      />
    </var-collapse-item>
  </var-collapse>

  <div v-if="items?.length" class="chapter-list">
    <button
      v-for="(item, index) in visibleItems"
      :key="item.id"
      type="button"
      :disabled="loading"
      @click="emit('open', item)"
    >
      <span v-if="numbered" class="chapter-number">{{ rowNumber(index) }}</span>
      <var-icon v-else-if="item.icon" :name="item.icon" />
      <span class="chapter-title">
        <strong>{{ item.title }}</strong>
        <small v-if="item.meta">{{ item.meta }}</small>
      </span>
      <var-chip v-if="item.unread" size="small">未读</var-chip>
    </button>
  </div>
  <div v-if="pageSize && items && items.length > pageSize" class="chapter-pagination">
    <span>第 {{ (page - 1) * pageSize + 1 }}–{{ Math.min(page * pageSize, items.length) }} 话</span>
    <var-pagination
      :max-pager-count="5"
      :size="pageSize"
      :total="items.length"
      :current="page"
      @update:current="page = Number($event)"
    />
  </div>
</template>
