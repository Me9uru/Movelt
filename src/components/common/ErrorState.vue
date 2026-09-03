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
    <var-result type="error" :title="title" :description="message">
      <template #footer>
        <var-button type="primary" :elevation="false" :loading="loading" @click="emit('retry')"
          >重试</var-button
        >
      </template>
    </var-result>
  </div>
</template>
