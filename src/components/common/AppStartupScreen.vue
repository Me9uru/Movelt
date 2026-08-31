<script setup lang="ts">
import appIconUrl from "../../../src-tauri/icons/128x128.png";
</script>

<template>
  <main
    class="startup-screen"
    role="status"
    aria-live="polite"
    aria-label="正在恢复登录状态"
  >
    <img class="startup-screen__icon" :src="appIconUrl" alt="" />
    <span class="startup-screen__spinner" aria-hidden="true"></span>
    <p>正在恢复登录状态…</p>
  </main>
</template>
