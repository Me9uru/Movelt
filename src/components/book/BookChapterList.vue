<script setup lang="ts" generic="T">
import { computed, ref } from "vue";
import type {
  BookChapterGroup,
  BookChapterItem,
} from "../../types/book";

interface BookChapterListProps<T> {
  items?: BookChapterItem<T>[];
  groups?: BookChapterGroup<T>[];
  loading?: boolean;
  pageSize?: number;
}

const props = withDefaults(
  defineProps<BookChapterListProps<T>>(),
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

const rowNumber = (index: number): string => {
  return String((page.value - 1) * (props.pageSize ?? 1) + index + 1).padStart(
    2,
    "0",
  );
}

const openChapter = (item: BookChapterItem<T>): void => {
  if (!props.loading) emit("open", item);
};

const handleChapterKeydown = (
  event: KeyboardEvent,
  item: BookChapterItem<T>,
): void => {
  if (event.key !== "Enter" && event.key !== " ") return;
  event.preventDefault();
  openChapter(item);
};
</script>

<template>
  <var-collapse
    v-if="groups?.length"
    v-model="activeSections"
    :divider="false"
    :elevation="false"
    :offset="false"
    class="catalogue catalogue-nested"
  >
    <var-collapse-item
      v-for="(group, index) in groups"
      :key="`${group.title}-${index}`"
      class="catalogue-section-item"
      :name="index"
    >
      <template #title>
        <div class="volume-title volume-title--section">
          <span class="volume-index">{{
            String(index + 1).padStart(2, "0")
          }}</span>
          <strong>{{ group.title }}</strong>
          <var-chip class="count-tag" size="small"
            >{{ group.count }} 话</var-chip
          >
        </div>
      </template>
      <div class="volume-content">
        <div class="volume-content-label">
          {{ group.groups?.length ? "分组与章节" : "章节" }}
        </div>
        <BookChapterList
          :groups="group.groups"
          :items="group.chapters"
          :loading="loading"
          :page-size="pageSize"
          v-memo="[group.groups, group.chapters]"
          @open="emit('open', $event)"
        />
      </div>
    </var-collapse-item>
  </var-collapse>

  <div v-if="items?.length" class="chapter-list">
    <var-cell
      v-for="(item, index) in visibleItems"
      :key="item.id"
      :title="item.title"
      :description="item.meta || undefined"
      :ripple="!loading"
      role="button"
      :tabindex="loading ? -1 : 0"
      :aria-disabled="loading"
      @click="openChapter(item)"
      @keydown="handleChapterKeydown($event, item)"
    >
      <template v-if="numbered || item.icon" #icon>
        <span v-if="numbered" class="chapter-number">{{ rowNumber(index) }}</span>
        <var-icon v-else :name="item.icon" />
      </template>
      <template #extra>
        <span class="chapter-extra">
          <var-chip v-if="item.unread" type="primary" size="small">
            未读
          </var-chip>
          <var-icon name="chevron-right" aria-hidden="true" />
        </span>
      </template>
    </var-cell>
  </div>
  <div
    v-if="pageSize && items && items.length > pageSize"
    class="chapter-pagination"
  >
    <span
      >第 {{ (page - 1) * pageSize + 1 }}–{{
        Math.min(page * pageSize, items.length)
      }}
      话</span
    >
    <var-pagination
      :max-pager-count="5"
      :size="pageSize"
      :total="items.length"
      :current="page"
      @update:current="page = Number($event)"
    />
  </div>
</template>
