<script setup lang="ts">
defineProps<{
  visible: boolean;
  label: string;
  inline?: boolean;
  contained?: boolean;
}>();
</script>

<template>
  <Teleport to="body" :disabled="inline">
    <Transition name="loading-fade">
      <div
        v-if="visible"
        class="loading-overlay"
        :class="{
          'loading-overlay--inline': inline,
          'loading-overlay--contained': contained,
        }"
        role="status"
        aria-live="polite"
        :aria-label="label"
      >
        <svg
          class="loading-overlay__icon"
          viewBox="-10 -10 50 50"
          aria-hidden="true"
        >
          <path
            class="path"
            d="M 30 15 L 28 17 M 25.61 25.61 A 15 15, 0, 0, 1, 15 30 A 15 15, 0, 1, 1, 27.99 7.5 L 15 15"
          />
        </svg>
      </div>
    </Transition>
  </Teleport>
</template>
