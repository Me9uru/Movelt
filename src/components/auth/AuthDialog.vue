<script setup lang="ts">
import { ref } from "vue";
import { sendRegisterEmail } from "../../services/auth";
import { useAuthStore } from "../../stores/auth";
import { showError } from "../../utils/error";

const visible = defineModel<boolean>("visible", { required: true });

const emit = defineEmits<{ authenticated: []; }>();
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

async function submitLogin() {
  if (!email.value || !password.value) return;
  submitting.value = true;
  try {
    await auth.login(email.value, password.value);
    emit("authenticated");
    visible.value = false;
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
    emit("authenticated");
    visible.value = false;
  } catch (error) {
    showError(error);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <el-dialog v-model="visible" width="min(420px, calc(100vw - 32px))" class="auth-dialog" modal-class="auth-dialog-mask"
    :close-on-click-modal="false" :close-on-press-escape="false" :show-close="false" append-to-body>
    <template #header>
      <div class="auth-dialog-title"><strong>Movel</strong></div>
    </template>
    <el-tabs v-model="authMode" stretch class="auth-tabs"><el-tab-pane label="登录" name="login" /><el-tab-pane label="注册"
        name="register" /></el-tabs>
    <el-form v-if="authMode === 'login'" @submit.prevent="submitLogin">
      <el-form-item label="邮箱"><el-input v-model="email" autocomplete="email" /></el-form-item>
      <el-form-item label="密码"><el-input v-model="password" type="password" autocomplete="current-password"
          show-password /></el-form-item>
      <el-button type="primary" :loading="submitting" native-type="submit">登录</el-button>
    </el-form>
    <el-form v-else @submit.prevent="submitRegistration">
      <el-form-item label="昵称"><el-input v-model="registerName" autocomplete="username" /></el-form-item>
      <el-form-item label="邮箱"><el-input v-model="email" autocomplete="email" /></el-form-item>
      <el-form-item label="验证码"><el-input v-model="registerCode" autocomplete="one-time-code"><template
            #append><el-button :loading="registerEmailSending"
              @click="sendRegistrationCode">发送验证码</el-button></template></el-input></el-form-item>
      <el-form-item label="密码"><el-input v-model="password" type="password" autocomplete="new-password"
          show-password /></el-form-item>
      <el-form-item label="确认密码"><el-input v-model="registerPasswordConfirmation" type="password"
          autocomplete="new-password" show-password /></el-form-item>
      <el-form-item label="邀请码（可选）"><el-input v-model="registerInviteCode" /></el-form-item>
      <el-button type="primary" :loading="submitting" native-type="submit">注册并登录</el-button>
    </el-form>
  </el-dialog>
</template>
