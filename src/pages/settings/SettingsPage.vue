<script setup lang="ts">
import { ArrowRight, RefreshRight, User } from "@element-plus/icons-vue";
import { useAuthStore } from "../../stores/auth";
import { showError } from "../../utils/error";

const emit = defineEmits<{
  login: [];
}>();
const auth = useAuthStore();

async function refreshProfile() {
  try {
    await auth.refreshProfile();
  } catch (error) {
    showError(error);
  }
}

async function logout() {
  try {
    await auth.logout();
  } catch (error) {
    showError(error);
  }
}
</script>

<template>
  <section class="settings-view">
    <section class="settings-group" aria-labelledby="account-settings-title">
      <h2 id="account-settings-title">账户</h2>
      <div class="settings-list">
        <button type="button" class="settings-row settings-row--account" @click="!auth.user && emit('login')">
          <el-avatar :size="48" :src="auth.user?.Avatar" :icon="User" />
          <span class="settings-row__content">
            <strong>{{ auth.user?.UserName ?? "登录 LightNovelShelf" }}</strong>
          </span>
          <el-icon v-if="!auth.user" class="settings-row__arrow"><ArrowRight /></el-icon>
        </button>

        <button v-if="auth.user" type="button" class="settings-row" :disabled="auth.refreshing" @click="refreshProfile">
          <el-icon class="settings-row__icon"><RefreshRight /></el-icon>
          <span class="settings-row__content"><strong>刷新账户资料</strong></span>
          <span v-if="auth.refreshing" class="settings-row__status">刷新中</span>
          <el-icon v-else class="settings-row__arrow"><ArrowRight /></el-icon>
        </button>

        <button v-if="auth.user" type="button" class="settings-row settings-row--danger" @click="logout">
          <span class="settings-row__content"><strong>退出登录</strong></span>
          <el-icon class="settings-row__arrow"><ArrowRight /></el-icon>
        </button>
      </div>
    </section>

    <section class="settings-group" aria-labelledby="about-settings-title">
      <h2 id="about-settings-title">关于</h2>
      <div class="settings-list">
        <a
          class="settings-row settings-row--link"
          href="https://github.com/Me9uru/Movelt"
          target="_blank"
          rel="noreferrer"
        >
          <span class="settings-row__content">
            <strong>开源地址</strong>
            <span class="settings-row__status">github.com/Me9uru/Movelt</span>
          </span>
          <el-icon class="settings-row__arrow"><ArrowRight /></el-icon>
        </a>
      </div>
    </section>
  </section>
</template>
