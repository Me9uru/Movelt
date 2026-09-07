<script setup lang="ts">
import { Snackbar } from "@varlet/ui";
import { ref } from "vue";
import { useRouter } from "vue-router";
import AccountProfileDialog from "../../components/settings/AccountProfileDialog.vue";
import type { LightNovelUser } from "../../domain/auth";
import { clearWebviewCache } from "../../services/settings";
import { useAuthStore } from "../../stores/auth";
import { showError } from "../../utils/error";

const auth = useAuthStore();
const router = useRouter();
const profileVisible = ref(false);
const loggingOut = ref(false);
const clearingCache = ref(false);

const openAccount = () => {
  if (auth.user) {
    profileVisible.value = true;
    return;
  }
  router.push({ name: "login" });
}

const updateUser = (user: LightNovelUser) => {
  auth.user = user;
}

const logout = async () => {
  if (loggingOut.value) return;
  loggingOut.value = true;
  try {
    await auth.logout();
  } catch (error) {
    showError(error);
  } finally {
    loggingOut.value = false;
  }
}

const clearImageCache = async () => {
  if (clearingCache.value) return;
  clearingCache.value = true;
  try {
    await clearWebviewCache();
    Snackbar.success("已清除 WebView 浏览数据");
  } catch (error) {
    showError(error, "清除 WebView 数据失败");
  } finally {
    clearingCache.value = false;
  }
}
</script>

<template>
  <section class="settings-view">
    <section class="settings-group" aria-labelledby="account-settings-title">
      <h2 id="account-settings-title">账户</h2>
      <div class="settings-list">
        <var-button text :elevation="false" class="settings-row settings-row--account" :aria-haspopup="auth.user ? 'dialog' : undefined" @click="openAccount">
          <var-avatar :size="40" :src="auth.user?.Avatar"><var-icon v-if="!auth.user?.Avatar" name="account-circle" /></var-avatar>
          <span class="settings-row__content">
            <strong>{{ auth.user?.UserName ?? "登录 LightNovelShelf" }}</strong>
            <span class="settings-row__status">{{ auth.user ? "查看个人信息与修改头像" : "登录后同步书架和阅读进度" }}</span>
          </span>
          <var-icon class="settings-row__arrow" name="chevron-right" aria-hidden="true" />
        </var-button>
        <var-button v-if="auth.user" text :elevation="false" class="settings-row settings-row--danger" :loading="loggingOut" :disabled="loggingOut" @click="logout">
          <var-icon class="settings-row__icon" name="arrow-left" aria-hidden="true" />
          <span class="settings-row__content"><strong>退出登录</strong></span>
          <var-icon class="settings-row__arrow" name="chevron-right" aria-hidden="true" />
        </var-button>
      </div>
    </section>

    <section class="settings-group" aria-labelledby="storage-settings-title">
      <h2 id="storage-settings-title">存储</h2>
      <div class="settings-list">
        <var-button text :elevation="false" class="settings-row settings-row--two-line" :loading="clearingCache" :disabled="clearingCache" @click="clearImageCache">
          <var-icon class="settings-row__icon" name="delete" aria-hidden="true" />
          <span class="settings-row__content">
            <strong>清除 WebView 数据</strong>
            <span class="settings-row__status">会清除图片缓存和阅读设置</span>
          </span>
          <var-icon class="settings-row__arrow" name="chevron-right" aria-hidden="true" />
        </var-button>
      </div>
    </section>

    <section class="settings-group" aria-labelledby="about-settings-title">
      <h2 id="about-settings-title">关于</h2>
      <div class="settings-list">
        <var-link
          class="settings-row settings-row--link settings-row--two-line"
          underline="none"
          href="https://github.com/Me9uru/Movelt"
          target="_blank"
          rel="noreferrer"
        >
          <span class="settings-row__content">
            <strong>开源地址</strong>
            <span class="settings-row__status">github.com/Me9uru/Movelt</span>
          </span>
          <var-icon class="settings-row__arrow" name="chevron-right" aria-hidden="true" />
        </var-link>
      </div>
    </section>

    <account-profile-dialog v-model:show="profileVisible" :user="auth.user" @updated="updateUser" />
  </section>
</template>
