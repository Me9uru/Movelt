<script setup lang="ts">
defineProps<{
  visible: boolean;
  label: string;
  inline?: boolean;
}>();
</script>

<template>
  <Teleport to="body" :disabled="inline">
    <Transition name="loading-fade">
      <div
        v-if="visible"
        class="loading-overlay"
        :class="{ 'loading-overlay--inline': inline }"
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

<style scoped>
.loading-overlay {
  position: fixed;
  inset: 0;
  z-index: 3000;
  display: grid;
  place-items: center;
  background: var(--el-mask-color);
  cursor: wait;
  -webkit-backdrop-filter: blur(4px) saturate(108%);
  backdrop-filter: blur(4px) saturate(108%);
}

.loading-overlay__icon {
  width: 48px;
  height: 48px;
  color: var(--color-primary);
  filter: drop-shadow(0 3px 8px rgb(39 120 186 / 18%));
  animation: loading-rotate 1.5s linear infinite;
}

.loading-overlay__icon .path {
  fill: none;
  stroke: currentcolor;
  stroke-width: 4px;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-dasharray: 90 150;
  animation: loading-dash 1.5s ease-in-out infinite;
}

.loading-overlay--inline {
  position: static;
  z-index: auto;
  min-height: 280px;
  background: transparent;
  cursor: wait;
  -webkit-backdrop-filter: none;
  backdrop-filter: none;
}

.loading-fade-enter-active,
.loading-fade-leave-active {
  transition: opacity 0.18s ease;
}

.loading-fade-enter-from,
.loading-fade-leave-to {
  opacity: 0;
}

@keyframes loading-rotate {
  to { transform: rotate(360deg); }
}

@keyframes loading-dash {
  0% { stroke-dasharray: 1 200; stroke-dashoffset: 0; }
  50% { stroke-dasharray: 90 150; stroke-dashoffset: -40px; }
  100% { stroke-dasharray: 90 150; stroke-dashoffset: -120px; }
}
</style>
