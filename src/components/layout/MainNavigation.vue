<script setup lang="ts">
import type { AppRouteName, LibraryRouteName } from "../../types/router";

defineProps<{ view: AppRouteName; }>();
const emit = defineEmits<{ navigate: [view: LibraryRouteName]; }>();
</script>

<template>
  <nav v-if="view === 'novels' || view === 'bookshelf' || view === 'manga' || view === 'settings'" class="view-dock" aria-label="主栏目">
    <button type="button" :class="{ active: view === 'novels' }" :aria-current="view === 'novels' ? 'page' : undefined" @click="emit('navigate', 'novels')"><var-icon name="notebook" /><span>小说</span></button>
    <button type="button" :class="{ active: view === 'manga' }" :aria-current="view === 'manga' ? 'page' : undefined" @click="emit('navigate', 'manga')"><var-icon name="image" /><span>漫画</span></button>
    <button type="button" :class="{ active: view === 'bookshelf' }" :aria-current="view === 'bookshelf' ? 'page' : undefined" @click="emit('navigate', 'bookshelf')"><var-icon name="bookmark" /><span>书架</span></button>
    <button type="button" :class="{ active: view === 'settings' }" :aria-current="view === 'settings' ? 'page' : undefined" @click="emit('navigate', 'settings')"><var-icon name="cog" /><span>设置</span></button>
  </nav>
</template>
