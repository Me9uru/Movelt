<script setup lang="ts">
defineProps<{
  title: string;
  message: string;
  loading?: boolean;
}>();

const emit = defineEmits<{ retry: [] }>();
</script>

<template>
  <div class="content-error-state">
    <el-result icon="error" :title="title" :sub-title="message">
      <template #extra>
        <el-button type="primary" :loading="loading" @click="emit('retry')">重试</el-button>
      </template>
    </el-result>
  </div>
</template>

<style scoped>
.content-error-state {
  display: grid;
  min-height: 300px;
  place-items: center;
}
</style>
