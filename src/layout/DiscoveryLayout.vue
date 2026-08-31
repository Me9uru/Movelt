<script setup lang="ts">
import AppSectionHeader from "../components/common/AppSectionHeader.vue";
import type { DiscoveryLayoutProps, DiscoveryTab } from "../types/discovery";

defineProps<DiscoveryLayoutProps>();

const emit = defineEmits<{
  "update:modelValue": [value: DiscoveryTab];
}>();
</script>

<template>
  <section class="discovery-view">
    <AppSectionHeader
      :model-value="modelValue"
      :tabs="tabs"
      :swipe="swipe"
      @update:model-value="emit('update:modelValue', $event)"
    />
    <slot />
  </section>
</template>
