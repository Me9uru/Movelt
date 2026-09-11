<script setup lang="ts">
import { ref, watch } from "vue";
import LoadingOverlay from "../common/LoadingOverlay.vue";

const props = withDefaults(
  defineProps<{
    coverUrl: string | null;
    title: string;
    fit?: "contain" | "cover" | "fill" | "none" | "scale-down";
  }>(),
  {
    fit: "cover",
  },
);

const loading = ref(Boolean(props.coverUrl));
const failed = ref(false);

watch(
  () => props.coverUrl,
  (coverUrl) => {
    loading.value = Boolean(coverUrl);
    failed.value = false;
  },
);

const handleLoad = (): void => {
  loading.value = false;
}

const handleError = (): void => {
  loading.value = false;
  failed.value = true;
}
</script>

<template>
  <div class="book-cover-root">
    <!-- 保留布局坐标，让懒加载只请求进入视口的封面。 -->
    <var-image
      v-if="coverUrl"
      :key="coverUrl"
      v-show="!failed"
      :style="{ opacity: loading ? 0 : 1 }"
      :src="coverUrl"
      :alt="title"
      :fit="fit"
      lazy
      @load="handleLoad"
      @error="handleError"
    />
    <LoadingOverlay
      :visible="loading"
      inline
      contained
      label="正在加载封面"
    />
    <span v-if="!coverUrl || failed" class="cover-placeholder">
      <var-icon name="bookmark" />
    </span>
  </div>
</template>
