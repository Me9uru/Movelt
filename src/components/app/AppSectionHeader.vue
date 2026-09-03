<script setup lang="ts" generic="T extends string">
import { ref } from "vue";
import AppSettingsDrawer from "../settings/AppSettingsDrawer.vue";
import AppTabs from "../common/AppTabs.vue";
import { useAuthStore } from "../../stores/auth";

interface AppSectionHeaderProps<T extends string> {
  tabs: readonly {
    readonly name: T;
    readonly label: string;
    readonly icon?: string;
  }[];
  modelValue: T;
  swipe?: boolean;
}

const props = withDefaults(defineProps<AppSectionHeaderProps<T>>(), {
  swipe: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: T];
}>();

const settingsDrawerVisible = ref(false);
const auth = useAuthStore();
let touchStart: { x: number; y: number } | null = null;

const selectTab = (name: string): void => {
  emit("update:modelValue", name as T);
};

const handleTouchStart = (event: TouchEvent): void => {
  if (!props.swipe || event.touches.length !== 1) {
    touchStart = null;
    return;
  }

  const touch = event.touches[0];
  touchStart = { x: touch.clientX, y: touch.clientY };
};

const handleTouchEnd = (event: TouchEvent): void => {
  if (!props.swipe || !touchStart || event.changedTouches.length !== 1) {
    touchStart = null;
    return;
  }

  const touch = event.changedTouches[0];
  const deltaX = touch.clientX - touchStart.x;
  const deltaY = touch.clientY - touchStart.y;
  touchStart = null;

  if (Math.abs(deltaX) < 50 || Math.abs(deltaX) <= Math.abs(deltaY) * 1.25)
    return;

  const allTabs = props.tabs.map((tab) => tab.name);
  const currentIndex = allTabs.indexOf(props.modelValue);
  const nextIndex = deltaX < 0 ? currentIndex + 1 : currentIndex - 1;
  const nextTab = allTabs[nextIndex];
  if (nextTab) selectTab(nextTab);
};
</script>

<template>
  <div class="content-tabs-shell">
    <var-button
      class="content-tabs-avatar"
      text
      round
      aria-label="打开设置"
      @click="settingsDrawerVisible = true"
    >
      <var-avatar :size="32" :src="auth.user?.Avatar">
        <var-icon v-if="!auth.user?.Avatar" name="account-circle" />
      </var-avatar>
    </var-button>
    <AppTabs
      :model-value="modelValue"
      :tabs="tabs"
      class="content-tabs"
      accessibility-label="页面栏目"
      @touchstart.passive="handleTouchStart"
      @touchend.passive="handleTouchEnd"
      @touchcancel="touchStart = null"
      @update:model-value="selectTab(String($event))"
    />
    <span class="content-tabs-end-spacer" aria-hidden="true"></span>
  </div>

  <AppSettingsDrawer v-model:show="settingsDrawerVisible" />
</template>
