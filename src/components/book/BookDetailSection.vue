<script setup lang="ts">
defineProps<{
  title: string;
  summary: string;
}>();
</script>

<template>
  <section class="catalogue-section">
    <div class="catalogue-heading">
      <h2>{{ title }}</h2>
      <p>{{ summary }}</p>
    </div>
    <slot />
  </section>
</template>
