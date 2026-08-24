<script setup lang="ts">
import { Collection } from "@element-plus/icons-vue";

withDefaults(defineProps<{
  coverUrl: string | null;
  title: string;
  fit?: "contain" | "cover" | "fill" | "none" | "scale-down";
}>(), {
  fit: "cover",
});
</script>

<template>
  <el-image v-if="coverUrl" :src="coverUrl" :alt="title" :fit="fit">
    <template #error>
      <span class="cover-placeholder"><el-icon><Collection /></el-icon></span>
    </template>
  </el-image>
  <span v-else class="cover-placeholder"><el-icon><Collection /></el-icon></span>
</template>
