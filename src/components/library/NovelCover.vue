<script setup lang="ts">
import { ref, watch } from "vue";
import { Collection } from "@element-plus/icons-vue";
import { getNovelCoverDataUrl } from "../../services/novel";
import { localEpubAssetUrl, localEpubSourceId } from "../../services/localEpub";

const props = withDefaults(defineProps<{
  source: string;
  novelId: string;
  title: string;
  coverUrl: string | null;
  fit?: "contain" | "cover" | "fill" | "none" | "scale-down";
}>(), {
  fit: "cover",
});

const resolvedUrl = ref<string | null>(null);

watch(
  () => [props.source, props.novelId, props.coverUrl] as const,
  async ([source, novelId, coverUrl], _, onCleanup) => {
    let active = true;
    onCleanup(() => {
      active = false;
    });
    resolvedUrl.value = null;
    if (!coverUrl) {
      return;
    }
    try {
      const url = source === localEpubSourceId
        ? await localEpubAssetUrl(novelId, coverUrl)
        : await getNovelCoverDataUrl(source, novelId);
      if (active) {
        resolvedUrl.value = url;
      }
    } catch {
      if (active) {
        resolvedUrl.value = null;
      }
    }
  },
  { immediate: true },
);
</script>

<template>
  <el-image v-if="resolvedUrl" :src="resolvedUrl" :alt="title" :fit="fit">
    <template #error>
      <div class="cover-placeholder"><el-icon><Collection /></el-icon></div>
    </template>
  </el-image>
  <div v-else class="cover-placeholder"><el-icon><Collection /></el-icon></div>
</template>
