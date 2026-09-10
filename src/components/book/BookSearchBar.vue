<script setup lang="ts">
import { computed } from "vue";
import type { BookSearchMode } from "../../domain/discovery";

interface BookSearchBarProps {
  modelValue: string;
  loading: boolean;
  searchMode?: BookSearchMode;
  showSubmit?: boolean;
  subjectLabel?: string;
}

const searchModes: { value: BookSearchMode; label: string; }[] = [
  { value: "title", label: "作品名" },
  { value: "author", label: "作者" },
  { value: "tags", label: "标签" },
];

const props = withDefaults(defineProps<BookSearchBarProps>(), {
  showSubmit: true,
  subjectLabel: "作品",
});

const emit = defineEmits<{
  clear: [];
  submit: [];
  "update:modelValue": [value: string];
  "update:searchMode": [value: BookSearchMode];
}>();

const placeholder = computed(() => {
  if (props.searchMode === "author") return "输入作者名称";
  if (props.searchMode === "tags") return "输入标签，多个标签用逗号分隔";
  return `输入${props.subjectLabel}名称`;
});

const ariaLabel = computed(() => {
  if (props.searchMode === "author") return "按作者搜索";
  if (props.searchMode === "tags") return "按标签搜索";
  return `按${props.subjectLabel}名搜索`;
});

const updateSearchMode = (value: BookSearchMode): void => {
  emit("update:searchMode", value);
};
</script>

<template>
  <form
    class="search-box book-search"
    role="search"
    @submit.prevent="emit('submit')"
  >
    <var-segmented-buttons
      v-if="searchMode"
      :model-value="searchMode"
      :options="searchModes"
      checkmark
      class="book-search-modes"
      aria-label="搜索方式"
      @update:model-value="updateSearchMode"
    />
    <div class="book-search-controls">
      <var-input
        :model-value="modelValue"
        variant="outlined"
        clearable
        maxlength="100"
        :placeholder="placeholder"
        :aria-label="ariaLabel"
        @update:model-value="emit('update:modelValue', $event)"
        @clear="emit('clear')"
      />
      <var-button
        v-if="showSubmit"
        class="book-search-submit"
        type="primary"
        icon-container
        round
        :elevation="false"
        native-type="submit"
        :loading="loading"
        aria-label="搜索"
        title="搜索"
        ><var-icon name="magnify"
      /></var-button>
    </div>
  </form>
</template>
