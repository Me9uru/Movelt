<script setup lang="ts">
import { onBeforeUnmount, onMounted } from "vue";
import SettingsPage from "../../pages/settings/SettingsPage.vue";

const props = defineProps<{ show: boolean }>();

const emit = defineEmits<{
  "update:show": [value: boolean];
}>();

const handleAndroidBack = (event: Event): void => {
  if (!props.show) return;
  event.preventDefault();
  emit("update:show", false);
};

onMounted(() => {
  window.addEventListener("movel:android-back", handleAndroidBack);
});

onBeforeUnmount(() => {
  window.removeEventListener("movel:android-back", handleAndroidBack);
});
</script>

<template>
  <var-popup
    :show="show"
    class="app-settings-drawer"
    overlay-class="app-settings-drawer-mask"
    position="left"
    :default-style="false"
    @update:show="emit('update:show', $event)"
  >
    <header class="app-settings-drawer__header">
      <strong>设置</strong>
      <var-button
        class="app-settings-drawer__close"
        text
        round
        aria-label="关闭设置"
        @click="emit('update:show', false)"
      >
        <var-icon name="close" />
      </var-button>
    </header>
    <SettingsPage />
  </var-popup>
</template>
