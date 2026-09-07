<script setup lang="ts" generic="T">
import { computed, ref, useId, watch } from "vue";
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
const groupId = useId();
watch(
  () => [props.items, props.pageSize],
  () => { page.value = 1; },
);
const activeSections = ref<number[]>([0]);

const toggleSection = (index: number): void => {
  if (props.loading) return;
  activeSections.value = activeSections.value.includes(index)
    ? activeSections.value.filter((value) => value !== index)
    : [...activeSections.value, index];
};

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
  <div v-if="groups?.length" class="catalogue catalogue-nested">
    <section
      v-for="(group, index) in groups"
      :key="`${group.title}-${index}`"
      class="catalogue-section-item"
    >
      <var-button
        text
        :elevation="false"
        class="volume-toggle"
        :disabled="loading"
        :aria-expanded="activeSections.includes(index)"
        :aria-controls="`${groupId}-${index}`"
        @click="toggleSection(index)"
      >
        <span class="volume-title">
          <span class="volume-index">{{ String(index + 1).padStart(2, "0") }}</span>
          <strong>{{ group.title }}</strong>
          <span class="volume-count">{{ group.count }} 话</span>
          <var-icon :name="activeSections.includes(index) ? 'chevron-up' : 'chevron-down'" aria-hidden="true" />
        </span>
      </var-button>
      <var-collapse-transition :expand="activeSections.includes(index)">
        <div :id="`${groupId}-${index}`" class="volume-content" :inert="!activeSections.includes(index)">
          <BookChapterList
            :groups="group.groups"
            :items="group.chapters"
            :loading="loading"
            :page-size="pageSize"
            @open="emit('open', $event)"
          />
          <p v-if="!group.count" class="chapter-empty">暂无章节</p>
        </div>
      </var-collapse-transition>
    </section>
  </div>

  <div v-if="items?.length" class="chapter-list" :aria-busy="loading">
    <var-cell
      v-for="(item, index) in visibleItems"
      :key="item.id"
      :title="item.title"
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
          <span v-if="item.meta" class="chapter-meta">{{ item.meta }}</span>
        </span>
      </template>
    </var-cell>
  </div>
  <nav
    aria-label="章节分页"
    v-if="pageSize && items && items.length > pageSize"
    class="chapter-pagination"
  >
    <span aria-live="polite"
      >第 {{ (page - 1) * pageSize + 1 }}–{{
        Math.min(page * pageSize, items.length)
      }}
      话</span
    >
    <var-pagination
      simple
      :elevation="false"
      :show-size-changer="false"
      :disabled="loading"
      :size="pageSize"
      :total="items.length"
      :current="page"
      @update:current="page = Number($event)"
    />
  </nav>
</template>
