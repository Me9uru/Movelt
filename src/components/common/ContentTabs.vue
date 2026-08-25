<script setup lang="ts" generic="T extends string">
import { nextTick, ref } from "vue";
import type { BookSearchMode } from "../../domain/search";
import BookSearchBar from "../book/BookSearchBar.vue";

const props = withDefaults(
  defineProps<{
    tabs: { name: T; label: string }[];
    modelValue: T;
    searchLabel: string;
    query: string;
    searchLoading: boolean;
    searchMode?: BookSearchMode;
    twoPrimary?: boolean;
    swipe?: boolean;
  }>(),
  {
    searchMode: undefined,
    twoPrimary: false,
    swipe: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: T];
  search: [];
  clear: [];
  "update:query": [value: string];
  "update:searchMode": [value: BookSearchMode];
}>();

const searchDialogVisible = ref(false);
const searchBar = ref<InstanceType<typeof BookSearchBar> | null>(null);
let touchStart: { x: number; y: number } | null = null;

function focusSearchInput(): void {
  const input = (
    searchBar.value?.$el as HTMLElement | undefined
  )?.querySelector<HTMLInputElement>("input");
  input?.focus();
}

function selectTab(name: string): void {
  emit("update:modelValue", name as T);
}

function submitSearch(): void {
  if (!props.query.trim()) {
    void nextTick(focusSearchInput);
    return;
  }

  emit("search");
  searchDialogVisible.value = false;
}

function handleTouchStart(event: TouchEvent): void {
  if (!props.swipe || event.touches.length !== 1) {
    touchStart = null;
    return;
  }

  const touch = event.touches[0];
  touchStart = { x: touch.clientX, y: touch.clientY };
}

function handleTouchEnd(event: TouchEvent): void {
  if (!props.swipe || !touchStart || event.changedTouches.length !== 1) {
    touchStart = null;
    return;
  }

  const touch = event.changedTouches[0];
  const deltaX = touch.clientX - touchStart.x;
  const deltaY = touch.clientY - touchStart.y;
  touchStart = null;

  if (Math.abs(deltaX) < 50 || Math.abs(deltaX) <= Math.abs(deltaY) * 1.25)
    return;

  const allTabs = props.tabs.map((tab) => tab.name);
  const currentIndex = allTabs.indexOf(props.modelValue);
  const nextIndex = deltaX < 0 ? currentIndex + 1 : currentIndex - 1;
  const nextTab = allTabs[nextIndex];
  if (nextTab) selectTab(nextTab);
}
</script>

<template>
  <div
    class="discovery-tabs-shell"
    :class="{ 'discovery-tabs--two-primary': twoPrimary }"
    @touchstart.passive="handleTouchStart"
    @touchend.passive="handleTouchEnd"
    @touchcancel="touchStart = null"
  >
    <var-tabs
      :active="modelValue"
      class="discovery-tabs"
      @update:active="selectTab(String($event))"
    >
      <var-tab v-for="tab in tabs" :key="tab.name" :name="tab.name">{{
        tab.label
      }}</var-tab>
    </var-tabs>
    <button
      class="library-search-trigger discovery-search-trigger"
      type="button"
      :aria-label="searchLabel"
      @click="searchDialogVisible = true"
    >
      <var-icon name="magnify" />
    </button>
  </div>

  <var-popup
    v-model:show="searchDialogVisible"
    class="library-search-dialog"
    overlay-class="library-search-mask"
    position="center"
    :default-style="false"
    @opened="focusSearchInput"
  >
    <BookSearchBar
      ref="searchBar"
      class="discovery-search"
      :model-value="query"
      :search-mode="searchMode"
      :loading="searchLoading"
      @update:model-value="emit('update:query', $event)"
      @update:search-mode="emit('update:searchMode', $event)"
      @clear="emit('clear')"
      @submit="submitSearch"
    />
  </var-popup>
</template>
