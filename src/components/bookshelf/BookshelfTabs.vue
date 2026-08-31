<script setup lang="ts" generic="T extends string">
import { nextTick, ref } from "vue";
import type { BookSearchMode } from "../../domain/search";
import AppSectionHeader from "../common/AppSectionHeader.vue";
import BookSearchBar from "../discovery/BookSearchBar.vue";

const props = withDefaults(
  defineProps<{
    tabs: readonly { readonly name: T; readonly label: string }[];
    modelValue: T;
    query: string;
    loading: boolean;
    searchMode?: BookSearchMode;
  }>(),
  { searchMode: undefined },
);

const emit = defineEmits<{
  "update:modelValue": [value: T];
  "update:query": [value: string];
  "update:searchMode": [value: BookSearchMode];
  search: [];
  clear: [];
}>();

const searchDialogVisible = ref(false);
const searchBar = ref<InstanceType<typeof BookSearchBar> | null>(null);

const focusSearchInput = (): void => {
  const input = (
    searchBar.value?.$el as HTMLElement | undefined
  )?.querySelector<HTMLInputElement>("input");
  input?.focus();
};

const submitSearch = (): void => {
  if (!props.query.trim()) {
    void nextTick(focusSearchInput);
    return;
  }

  emit("search");
  searchDialogVisible.value = false;
};
</script>

<template>
  <AppSectionHeader
    :model-value="modelValue"
    :tabs="tabs"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <template #trailing>
      <var-button
        class="library-search-trigger"
        text
        aria-label="搜索书架"
        @click="searchDialogVisible = true"
      >
        <var-icon name="magnify" />
      </var-button>
    </template>
  </AppSectionHeader>

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
      :loading="loading"
      @update:model-value="emit('update:query', $event)"
      @update:search-mode="emit('update:searchMode', $event)"
      @clear="emit('clear')"
      @submit="submitSearch"
    />
  </var-popup>
</template>
