<script setup lang="ts">
import { Snackbar } from "@varlet/ui";
import { useRouter } from "vue-router";
import { clearWebviewCache } from "../../services/settings";
import { useAuthStore } from "../../stores/auth";
import { showError } from "../../utils/error";

const auth = useAuthStore();
const router = useRouter();

async function logout() {
  try {
    await auth.logout();
  } catch (error) {
    showError(error);
  }
}

async function clearImageCache() {
  try {
    await clearWebviewCache();
    Snackbar.success("已清除 WebView 浏览数据");
  } catch (error) {
    showError(error, "清除图片缓存失败");
  }
}
</script>

<template>
  <section class="settings-view">
    <section class="settings-group" aria-labelledby="account-settings-title">
      <h2 id="account-settings-title">账户</h2>
      <div class="settings-list">
        <var-cell class="settings-row settings-row--account" ripple @click="!auth.user && router.push({ name: 'login' })">
          <template #icon><var-avatar :size="48" :src="auth.user?.Avatar" /></template>
          <span class="settings-row__content">
            <strong>{{ auth.user?.UserName ?? "登录 LightNovelShelf" }}</strong>
          </span>
          <template v-if="!auth.user" #extra><var-icon class="settings-row__arrow" name="chevron-right" /></template>
        </var-cell>
        <var-cell v-if="auth.user" class="settings-row settings-row--danger" ripple @click="logout">
          <span class="settings-row__content"><strong>退出登录</strong></span>
          <template #extra><var-icon class="settings-row__arrow" name="chevron-right" /></template>
        </var-cell>
      </div>
    </section>

    <section class="settings-group" aria-labelledby="storage-settings-title">
      <h2 id="storage-settings-title">存储</h2>
      <div class="settings-list">
        <var-cell class="settings-row" icon="delete" ripple @click="clearImageCache">
          <span class="settings-row__content">
            <strong>清除 WebView 数据</strong>
            <span class="settings-row__status">会清除图片缓存和阅读设置</span>
          </span>
          <template #extra><var-icon class="settings-row__arrow" name="chevron-right" /></template>
        </var-cell>
      </div>
    </section>

    <section class="settings-group" aria-labelledby="about-settings-title">
      <h2 id="about-settings-title">关于</h2>
      <div class="settings-list">
        <var-link
          class="settings-row settings-row--link"
          underline="none"
          href="https://github.com/Me9uru/Movelt"
          target="_blank"
          rel="noreferrer"
        >
          <span class="settings-row__content">
            <strong>开源地址</strong>
            <span class="settings-row__status">github.com/Me9uru/Movelt</span>
          </span>
          <var-icon class="settings-row__arrow" name="chevron-right" />
        </var-link>
      </div>
    </section>
  </section>
</template>
