<script setup lang="ts" generic="T extends string">
import { nextTick, ref, watch } from "vue";

interface VarTabsInstance {
  resize: () => void;
}

interface AppTab<T> {
  name: T;
  label: string;
  icon?: string;
}

interface AppTabsProps<T> {
  modelValue: T;
  tabs: readonly AppTab<T>[];
  accessibilityLabel: string;
  variant?: "primary" | "secondary";
  stretch?: boolean;
}

const props = withDefaults(defineProps<AppTabsProps<T>>(), {
  variant: "primary",
  stretch: true,
});

const emit = defineEmits<{
  "update:modelValue": [value: T];
}>();
const tabsRef = ref<VarTabsInstance | null>(null);

watch(
  () => props.modelValue,
  async () => {
    await nextTick();
    requestAnimationFrame(() => tabsRef.value?.resize());
  },
  { flush: "post" },
);

const selectTab = (name: string | number): void => {
  emit("update:modelValue", String(name) as T);
};

const handleKeydown = (
  event: KeyboardEvent,
  currentIndex: number,
): void => {
  let nextIndex: number | undefined;
  if (event.key === "ArrowRight" || event.key === "ArrowDown") {
    nextIndex = (currentIndex + 1) % props.tabs.length;
  } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
    nextIndex = (currentIndex - 1 + props.tabs.length) % props.tabs.length;
  } else if (event.key === "Home") {
    nextIndex = 0;
  } else if (event.key === "End") {
    nextIndex = props.tabs.length - 1;
  }

  if (nextIndex === undefined) return;
  event.preventDefault();
  const nextTab = props.tabs[nextIndex];
  if (!nextTab) return;

  emit("update:modelValue", nextTab.name);
  const tabElements = (event.currentTarget as HTMLElement).parentElement
    ?.querySelectorAll<HTMLElement>("[role='tab']");
  tabElements?.[nextIndex]?.focus();
};
</script>

<template>
  <var-tabs
    ref="tabsRef"
    class="app-tabs"
    :class="[
      `app-tabs--${variant}`,
      { 'app-tabs--stretch': stretch },
    ]"
    :active="modelValue"
    :indicator-size="variant === 'primary' ? 3 : 2"
    active-color="var(--color-primary-deep)"
    inactive-color="var(--color-text-muted)"
    indicator-color="var(--color-primary)"
    role="tablist"
    :aria-label="accessibilityLabel"
    @update:active="selectTab"
  >
    <var-tab
      v-for="(tab, index) in tabs"
      :key="tab.name"
      :name="tab.name"
      role="tab"
      :aria-selected="tab.name === modelValue"
      :tabindex="tab.name === modelValue ? 0 : -1"
      @keydown="handleKeydown($event, index)"
    >
      <var-icon
        v-if="tab.icon"
        class="app-tabs__icon"
        :name="tab.icon"
        aria-hidden="true"
      />
      {{ tab.label }}
    </var-tab>
  </var-tabs>
</template>
