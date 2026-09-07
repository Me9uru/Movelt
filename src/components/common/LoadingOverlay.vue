<script setup lang="ts">
defineProps<{
  visible: boolean;
  label: string;
  inline?: boolean;
  contained?: boolean;
}>();
</script>

<template>
  <Teleport to="body" :disabled="inline || contained">
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
        <var-loading type="circle" :radius="20" aria-hidden="true" />
      </div>
    </Transition>
  </Teleport>
</template>
