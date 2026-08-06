<script setup lang="ts">
import { computed } from "vue";
import type { ReadingProgress } from "../../domain/library";

const props = defineProps<{
  progress: ReadingProgress;
}>();

const percentage = computed(() =>
  Math.round(Math.min(1, Math.max(0, props.progress.bookLocation)) * 100),
);
</script>

<template>
  <div class="reading-progress">
    <div class="reading-progress-meta">
      <span>{{ progress.documentTitle }}</span>
      <strong>全书 {{ percentage }}%</strong>
    </div>
    <el-progress
      :percentage="percentage"
      :stroke-width="4"
      :show-text="false"
      :aria-label="`全书阅读进度 ${percentage}%`"
    />
  </div>
</template>
