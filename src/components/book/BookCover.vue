<script setup lang="ts">
withDefaults(defineProps<{
  coverUrl: string | null;
  title: string;
  fit?: "contain" | "cover" | "fill" | "none" | "scale-down";
}>(), {
  fit: "cover",
});
</script>

<template>
  <var-image v-if="coverUrl" :src="coverUrl" :alt="title" :fit="fit">
    <template #error>
      <span class="cover-placeholder"><var-icon name="bookmark" /></span>
    </template>
  </var-image>
  <span v-else class="cover-placeholder"><var-icon name="bookmark" /></span>
</template>
