<script setup lang="ts">
import { computed } from "vue";
import type { BookSearchMode } from "../../domain/search";
import type { BookSearchBarProps } from "../../types/discovery";

const searchModes: { value: BookSearchMode; label: string }[] = [
  { value: "title", label: "作品名" },
  { value: "author", label: "作者" },
  { value: "tags", label: "标签" },
];

const props = defineProps<BookSearchBarProps>();

const emit = defineEmits<{
  clear: [];
  submit: [];
  "update:modelValue": [value: string];
  "update:searchMode": [value: BookSearchMode];
}>();

const placeholder = computed(() => {
  if (props.placeholder) return props.placeholder;
  if (props.searchMode === "author") return "输入作者名称";
  if (props.searchMode === "tags") return "输入标签，多个标签用逗号分隔";
  return "输入作品名";
});

const ariaLabel = computed(() => {
  if (props.ariaLabel) return props.ariaLabel;
  if (props.searchMode === "author") return "按作者搜索";
  if (props.searchMode === "tags") return "按标签搜索";
  return "按作品名搜索";
});
</script>

<template>
  <form
    class="search-box book-search"
    role="search"
    @submit.prevent="emit('submit')"
  >
    <div
      v-if="searchMode"
      class="book-search-modes"
      role="tablist"
      aria-label="搜索方式"
    >
      <var-button
        v-for="mode in searchModes"
        :key="mode.value"
        text
        class="book-search-mode"
        :class="{ 'book-search-mode--active': searchMode === mode.value }"
        role="tab"
        :aria-selected="searchMode === mode.value"
        @click="emit('update:searchMode', mode.value)"
      >
        {{ mode.label }}
      </var-button>
    </div>
    <div class="book-search-controls">
      <var-input
        :model-value="modelValue"
        clearable
        maxlength="100"
        :placeholder="placeholder"
        :aria-label="ariaLabel"
        @update:model-value="emit('update:modelValue', $event)"
        @clear="emit('clear')"
      />
      <var-button
        type="primary"
        native-type="submit"
        :loading="loading"
        aria-label="搜索"
        title="搜索"
        ><var-icon name="magnify"
      /></var-button>
    </div>
  </form>
</template>
