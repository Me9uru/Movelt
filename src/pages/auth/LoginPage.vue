<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import type { LoginInput, RegisterInput } from "../../domain/auth";
import { sendRegisterEmail } from "../../services/auth";
import { useAuthStore } from "../../stores/auth";
import { showError } from "../../utils/error";
import AppTabs from "../../components/common/AppTabs.vue";

const route = useRoute();
const router = useRouter();
const auth = useAuthStore();

const authMode = ref<"login" | "register">("login");
const authTabs = [
  { name: "login", label: "登录" },
  { name: "register", label: "注册" },
] as const;
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

const finishAuthentication = async () => {
  await router.replace(redirectTarget.value);
}

const submitLogin = async () => {
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

const sendRegistrationCode = async () => {
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

const submitRegistration = async () => {
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
      <AppTabs
        v-model="authMode"
        class="auth-tabs"
        :tabs="authTabs"
        accessibility-label="账户操作"
      />
      <form v-if="authMode === 'login'" @submit.prevent="submitLogin">
        <label>邮箱<var-input v-model="credentials.email" variant="outlined" autocomplete="email" /></label>
        <label>密码<var-input v-model="credentials.password" variant="outlined" type="password" autocomplete="current-password" /></label>
        <var-button class="auth-submit-button" type="primary" :elevation="false" :loading="submitting" native-type="submit">登录</var-button>
      </form>
      <form v-else @submit.prevent="submitRegistration">
        <label>昵称<var-input v-model="registration.userName" variant="outlined" autocomplete="username" /></label>
        <label>邮箱<var-input v-model="credentials.email" variant="outlined" autocomplete="email" /></label>
        <label>验证码
          <span class="auth-code-row">
            <var-input v-model="registration.code" variant="outlined" autocomplete="one-time-code" />
            <var-button class="auth-code-button" type="primary" tonal :loading="registerEmailSending" @click="sendRegistrationCode">发送验证码</var-button>
          </span>
        </label>
        <label>密码<var-input v-model="credentials.password" variant="outlined" type="password" autocomplete="new-password" /></label>
        <label>确认密码<var-input v-model="registerPasswordConfirmation" variant="outlined" type="password" autocomplete="new-password" /></label>
        <label>邀请码（可选）<var-input v-model="registration.inviteCode" variant="outlined" /></label>
        <var-button class="auth-submit-button" type="primary" :elevation="false" :loading="submitting" native-type="submit">注册并登录</var-button>
      </form>
    </div>
  </section>
</template>
