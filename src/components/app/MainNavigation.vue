<script setup lang="ts">
import { computed } from "vue";
import type { AppRouteName, NavigationRouteName } from "../../types/router";

const props = defineProps<{ view: AppRouteName }>();
const emit = defineEmits<{ navigate: [view: NavigationRouteName] }>();
const navItems = [
  { name: "novels" as const, icon: "notebook", label: "小说" },
  { name: "comic" as const, icon: "image", label: "漫画" },
  { name: "bookshelf" as const, icon: "bookmark", label: "书架" },
] as const;
const shouldShow = computed(() => {
  return navItems.some((item) => item.name === props.view);
});
const navigate = (view: string | number): void => {
  emit("navigate", view as NavigationRouteName);
};
</script>

<template>
  <var-bottom-navigation
    v-if="shouldShow"
    class="view-dock"
    :active="props.view"
    fixed
    variant
    aria-label="主栏目"
    @update:active="navigate"
  >
    <var-bottom-navigation-item
      v-for="item in navItems"
      :key="item.name"
      :name="item.name"
      :icon="item.icon"
      :label="item.label"
    />
  </var-bottom-navigation>
</template>
