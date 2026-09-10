<script setup lang="ts">
defineProps<{
  label: string;
  previousDisabled: boolean;
  nextDisabled: boolean;
}>();
const emit = defineEmits<{ previous: []; next: []; }>();
</script>

<template>
  <nav class="page-controls" aria-label="分页状态与章节导航">
    <var-button text round :elevation="false" aria-label="上一页" :disabled="previousDisabled" @click="emit('previous')">
      <var-icon name="chevron-left" />
    </var-button>
    <span class="page-status" role="status">{{ label }}</span>
    <var-button text round :elevation="false" aria-label="下一页" :disabled="nextDisabled" @click="emit('next')">
      <var-icon name="chevron-right" />
    </var-button>
  </nav>
</template>
