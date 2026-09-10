<script setup lang="ts">
import { ref, watch } from "vue";
const props = defineProps<{ url: string; index: number; }>();
const emit = defineEmits<{ load: []; }>();
const loaded = ref(false);
const failed = ref(false);
watch(() => props.url, () => { loaded.value = false; failed.value = false; });
</script>

<template>
  <div class="comic-reader-page" :class="{ 'comic-reader-page--loaded': loaded }" :data-page-index="index">
    <img
      v-if="url && !failed"
      class="comic-reader-page-image"
      :src="url"
      :alt="`第 ${index + 1} 页`"
      @load="loaded = true; emit('load')"
      @error="failed = true"
    />
    <var-button v-else-if="failed" text :elevation="false" @click.stop="failed = false">重新加载第 {{ index + 1 }} 页</var-button>
    <span v-else class="comic-reader-page-placeholder">加载第 {{ index + 1 }} 页…</span>
  </div>
</template>
