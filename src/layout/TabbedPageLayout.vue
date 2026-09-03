<script setup lang="ts" generic="T extends string">
import AppSectionHeader from "../components/app/AppSectionHeader.vue";

interface TabbedPageLayoutProps<T extends string> {
  tabs: readonly {
    readonly name: T;
    readonly label: string;
    readonly icon?: string;
  }[];
  modelValue: T;
  swipe?: boolean;
}

defineProps<TabbedPageLayoutProps<T>>();

const emit = defineEmits<{
  "update:modelValue": [value: T];
}>();
</script>

<template>
  <section class="tabbed-page">
    <AppSectionHeader
      :model-value="modelValue"
      :tabs="tabs"
      :swipe="swipe"
      @update:model-value="emit('update:modelValue', $event)"
    />
    <slot />
  </section>
</template>
