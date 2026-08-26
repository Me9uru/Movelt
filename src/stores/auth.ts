import { computed, ref } from "vue";
import { defineStore } from "pinia";
import {
  login as loginRequest,
  logout as logoutRequest,
  register as registerRequest,
  restoreUser,
  signIn as signInRequest,
} from "../services/auth";
import type { LightNovelUser, LoginInput, RegisterInput } from "../domain/auth";

export const useAuthStore = defineStore("auth", () => {
  const user = ref<LightNovelUser | null>(null);
  const restoring = ref(false);
  const isAuthenticated = computed(() => user.value !== null);

  async function restore() {
    restoring.value = true;
    try {
      user.value = await restoreUser();
      void signInIfNeeded();
      return user.value;
    } finally {
      restoring.value = false;
    }
  }

  async function login(input: LoginInput) {
    user.value = await loginRequest(input);
    void signInIfNeeded();
    return user.value;
  }

  async function register(input: RegisterInput) {
    user.value = await registerRequest(input);
    void signInIfNeeded();
    return user.value;
  }

  async function signInIfNeeded() {
    if (user.value?.Growth?.TodaySigned !== false) return false;
    try {
      user.value = await signInRequest();
      return true;
    } catch {
      // 自动签到不应妨碍登录或恢复会话；下次启动时会再次尝试。
      return false;
    }
  }

  async function logout() {
    await logoutRequest();
    user.value = null;
  }

  function expire() {
    user.value = null;
  }

  return {
    user,
    restoring,
    isAuthenticated,
    restore,
    login,
    register,
    signInIfNeeded,
    logout,
    expire,
  };
});
