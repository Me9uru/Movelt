<script setup lang="ts">
import { Search } from "@element-plus/icons-vue";

defineProps<{
  modelValue: string;
  loading: boolean;
  placeholder?: string;
  ariaLabel?: string;
}>();

const emit = defineEmits<{
  clear: [];
  submit: [];
  "update:modelValue": [value: string];
}>();
</script>

<template>
  <form class="search-box book-search" role="search" @submit.prevent="emit('submit')">
    <el-input
      :model-value="modelValue"
      :prefix-icon="Search"
      clearable
      maxlength="100"
      :placeholder="placeholder || '输入书名'"
      :aria-label="ariaLabel || '按书名搜索'"
      @update:model-value="emit('update:modelValue', $event)"
      @clear="emit('clear')"
    />
    <el-button
      type="primary"
      native-type="submit"
      :icon="Search"
      :loading="loading"
      aria-label="搜索"
      title="搜索"
    />
  </form>
</template>
