<script setup lang="ts">
defineProps<{
  icon?: string;
  title: string;
  description?: string;
}>();
</script>

<template>
  <div class="app-empty-state">
    <div v-if="icon" class="app-empty-state__icon" aria-hidden="true">
      <var-icon :name="icon" />
    </div>
    <h2>{{ title }}</h2>
    <p v-if="description">{{ description }}</p>
  </div>
</template>
