<script setup lang="ts">
import { Collection } from "@element-plus/icons-vue";

const props = withDefaults(defineProps<{
  source: string;
  novelId: string;
  title: string;
  coverUrl: string | null;
  fit?: "contain" | "cover" | "fill" | "none" | "scale-down";
}>(), {
  fit: "cover",
});

</script>

<template>
  <el-image v-if="coverUrl" :src="coverUrl" :alt="title" :fit="fit">
    <template #error>
      <div class="cover-placeholder"><el-icon><Collection /></el-icon></div>
    </template>
  </el-image>
  <div v-else class="cover-placeholder"><el-icon><Collection /></el-icon></div>
</template>
