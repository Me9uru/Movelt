<script setup lang="ts">
withDefaults(
  defineProps<{
    content: string | null | undefined;
    fallback?: string;
  }>(),
  {
    fallback: "暂无简介。",
  },
);
</script>

<template>
  <div class="work-description" v-html="content || fallback" />
</template>
