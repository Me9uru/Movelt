<script setup lang="ts">
import type { AppRouteName, LibraryRouteName } from "../types/router";

defineProps<{ view: AppRouteName; }>();
const emit = defineEmits<{ navigate: [view: LibraryRouteName]; }>();

function navigate(view: string | number): void {
  emit("navigate", view as LibraryRouteName);
}
</script>

<template>
  <var-bottom-navigation
    v-if="view === 'novels' || view === 'bookshelf' || view === 'manga' || view === 'settings'"
    class="view-dock"
    :active="view"
    fixed
    variant
    aria-label="主栏目"
    @update:active="navigate"
  >
    <var-bottom-navigation-item name="novels" icon="notebook" label="小说" />
    <var-bottom-navigation-item name="manga" icon="image" label="漫画" />
    <var-bottom-navigation-item name="bookshelf" icon="bookmark" label="书架" />
    <var-bottom-navigation-item name="settings" icon="cog" label="设置" />
  </var-bottom-navigation>
</template>
