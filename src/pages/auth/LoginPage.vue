<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import type { LoginInput, RegisterInput } from "../../domain/auth";
import { sendRegisterEmail } from "../../services/auth";
import { useAuthStore } from "../../stores/auth";
import { showError } from "../../utils/error";

const route = useRoute();
const router = useRouter();
const auth = useAuthStore();

const authMode = ref<"login" | "register">("login");
const credentials = reactive<LoginInput>({
  email: "",
  password: "",
});
const submitting = ref(false);
const registration = reactive<Omit<RegisterInput, keyof LoginInput>>({
  userName: "",
  code: "",
  inviteCode: "",
});
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
  if (!credentials.email || !credentials.password) return;
  submitting.value = true;
  try {
    await auth.login(credentials);
    await finishAuthentication();
  } catch (error) {
    showError(error);
  } finally {
    submitting.value = false;
  }
}

async function sendRegistrationCode() {
  if (!credentials.email) return;
  registerEmailSending.value = true;
  try {
    await sendRegisterEmail(credentials.email);
  } catch (error) {
    showError(error);
  } finally {
    registerEmailSending.value = false;
  }
}

async function submitRegistration() {
  if (!registration.userName || !credentials.email || !credentials.password || !registration.code) return;
  if (credentials.password !== registerPasswordConfirmation.value) {
    showError("两次输入的密码不一致");
    return;
  }
  submitting.value = true;
  try {
    await auth.register({ ...credentials, ...registration });
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
        <span>Ciallo～(∠・ω< )⌒★</span>
      </div>
      <var-tabs :active="authMode" class="auth-tabs" @update:active="authMode = $event as 'login' | 'register'">
        <var-tab name="login">登录</var-tab>
        <var-tab name="register">注册</var-tab>
      </var-tabs>
      <form v-if="authMode === 'login'" @submit.prevent="submitLogin">
        <label>邮箱<var-input v-model="credentials.email" autocomplete="email" /></label>
        <label>密码<var-input v-model="credentials.password" type="password" autocomplete="current-password" /></label>
        <var-button type="primary" :loading="submitting" native-type="submit">登录</var-button>
      </form>
      <form v-else @submit.prevent="submitRegistration">
        <label>昵称<var-input v-model="registration.userName" autocomplete="username" /></label>
        <label>邮箱<var-input v-model="credentials.email" autocomplete="email" /></label>
        <label>验证码
          <span class="auth-code-row">
            <var-input v-model="registration.code" autocomplete="one-time-code" />
            <var-button :loading="registerEmailSending" @click="sendRegistrationCode">发送验证码</var-button>
          </span>
        </label>
        <label>密码<var-input v-model="credentials.password" type="password" autocomplete="new-password" /></label>
        <label>确认密码<var-input v-model="registerPasswordConfirmation" type="password" autocomplete="new-password" /></label>
        <label>邀请码（可选）<var-input v-model="registration.inviteCode" /></label>
        <var-button type="primary" :loading="submitting" native-type="submit">注册并登录</var-button>
      </form>
    </div>
  </section>
</template>
