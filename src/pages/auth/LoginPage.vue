<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { sendRegisterEmail } from "../../services/auth";
import { useAuthStore } from "../../stores/auth";
import { showError } from "../../utils/error";

const route = useRoute();
const router = useRouter();
const auth = useAuthStore();

const authMode = ref<"login" | "register">("login");
const email = ref("");
const password = ref("");
const submitting = ref(false);
const registerName = ref("");
const registerCode = ref("");
const registerInviteCode = ref("");
const registerPasswordConfirmation = ref("");
const registerEmailSending = ref(false);
const redirectTarget = computed(() => {
  const redirect = route.query.redirect;
  return typeof redirect === "string" && redirect.startsWith("/") && !redirect.startsWith("//")
    ? redirect
    : "/novels";
});

async function finishAuthentication() {
  await router.replace(redirectTarget.value);
}

async function submitLogin() {
  if (!email.value || !password.value) return;
  submitting.value = true;
  try {
    await auth.login(email.value, password.value);
    await finishAuthentication();
  } catch (error) {
    showError(error);
  } finally {
    submitting.value = false;
  }
}

async function sendRegistrationCode() {
  if (!email.value) return;
  registerEmailSending.value = true;
  try {
    await sendRegisterEmail(email.value);
  } catch (error) {
    showError(error);
  } finally {
    registerEmailSending.value = false;
  }
}

async function submitRegistration() {
  if (!registerName.value || !email.value || !password.value || !registerCode.value) return;
  if (password.value !== registerPasswordConfirmation.value) {
    showError("两次输入的密码不一致");
    return;
  }
  submitting.value = true;
  try {
    await auth.register(registerName.value, email.value, password.value, registerCode.value, registerInviteCode.value);
    await finishAuthentication();
  } catch (error) {
    showError(error);
  } finally {
    submitting.value = false;
  }
}

onMounted(() => {
  if (auth.user) void finishAuthentication();
});
</script>

<template>
  <section class="auth-page" aria-labelledby="auth-page-title">
    <div class="auth-card">
      <div class="auth-page-title">
        <strong id="auth-page-title">Movel</strong>
        <span>登录 LightNovelShelf，继续阅读</span>
      </div>
      <el-tabs v-model="authMode" stretch class="auth-tabs">
        <el-tab-pane label="登录" name="login" />
        <el-tab-pane label="注册" name="register" />
      </el-tabs>
      <el-form v-if="authMode === 'login'" @submit.prevent="submitLogin">
        <el-form-item label="邮箱"><el-input v-model="email" autocomplete="email" /></el-form-item>
        <el-form-item label="密码"><el-input v-model="password" type="password" autocomplete="current-password" show-password /></el-form-item>
        <el-button type="primary" :loading="submitting" native-type="submit">登录</el-button>
      </el-form>
      <el-form v-else @submit.prevent="submitRegistration">
        <el-form-item label="昵称"><el-input v-model="registerName" autocomplete="username" /></el-form-item>
        <el-form-item label="邮箱"><el-input v-model="email" autocomplete="email" /></el-form-item>
        <el-form-item label="验证码">
          <el-input v-model="registerCode" autocomplete="one-time-code">
            <template #append><el-button :loading="registerEmailSending" @click="sendRegistrationCode">发送验证码</el-button></template>
          </el-input>
        </el-form-item>
        <el-form-item label="密码"><el-input v-model="password" type="password" autocomplete="new-password" show-password /></el-form-item>
        <el-form-item label="确认密码"><el-input v-model="registerPasswordConfirmation" type="password" autocomplete="new-password" show-password /></el-form-item>
        <el-form-item label="邀请码（可选）"><el-input v-model="registerInviteCode" /></el-form-item>
        <el-button type="primary" :loading="submitting" native-type="submit">注册并登录</el-button>
      </el-form>
    </div>
  </section>
</template>
