<script setup lang="ts">
import { Collection, Compass, Picture, Setting } from "@element-plus/icons-vue";
import type { AppRouteName, LibraryRouteName } from "../../router";

defineProps<{ view: AppRouteName; bookCount: number; }>();
const emit = defineEmits<{ navigate: [view: LibraryRouteName]; }>();
</script>

<template>
  <nav v-if="view === 'novels' || view === 'bookshelf' || view === 'manga' || view === 'settings'" class="view-dock" aria-label="主栏目">
    <button type="button" :class="{ active: view === 'novels' }" :aria-current="view === 'novels' ? 'page' : undefined" @click="emit('navigate', 'novels')"><el-icon><Compass /></el-icon><span>小说</span></button>
    <button type="button" :class="{ active: view === 'manga' }" :aria-current="view === 'manga' ? 'page' : undefined" @click="emit('navigate', 'manga')"><el-icon><Picture /></el-icon><span>漫画</span></button>
    <button type="button" :class="{ active: view === 'bookshelf' }" :aria-current="view === 'bookshelf' ? 'page' : undefined" @click="emit('navigate', 'bookshelf')"><el-icon><Collection /></el-icon><span>书架</span><small v-if="bookCount">{{ bookCount }}</small></button>
    <button type="button" :class="{ active: view === 'settings' }" :aria-current="view === 'settings' ? 'page' : undefined" @click="emit('navigate', 'settings')"><el-icon><Setting /></el-icon><span>设置</span></button>
  </nav>
</template>
