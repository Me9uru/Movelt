<script setup lang="ts">
import { Search } from "@element-plus/icons-vue";

defineProps<{
  modelValue: string;
  loading: boolean;
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
      placeholder="输入书名"
      aria-label="按书名搜索"
      @update:model-value="emit('update:modelValue', $event)"
      @clear="emit('clear')"
    />
    <el-button type="primary" native-type="submit" :loading="loading">
      搜索
    </el-button>
  </form>
</template>
