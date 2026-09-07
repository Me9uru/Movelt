<script setup lang="ts">
defineProps<{ title: string; settingsDisabled?: boolean }>();
const emit = defineEmits<{ back: []; settings: [] }>();
</script>

<template>
  <header class="reader-toolbar">
    <var-button text round :elevation="false" aria-label="返回目录" @click="emit('back')">
      <var-icon name="arrow-left" aria-hidden="true" />
    </var-button>
    <span class="reader-toolbar__title" :title="title">{{ title }}</span>
    <var-button
      text
      round
      :elevation="false"
      :disabled="settingsDisabled"
      aria-label="打开阅读设置"
      aria-haspopup="dialog"
      @click="emit('settings')"
    >
      <var-icon name="cog-outline" aria-hidden="true" />
    </var-button>
  </header>
</template>
