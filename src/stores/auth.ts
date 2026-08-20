import { computed, ref } from "vue";
import { defineStore } from "pinia";
import {
  login as loginRequest,
  logout as logoutRequest,
  register as registerRequest,
  restoreUser,
  type LightNovelUser,
} from "../services/auth";

export const useAuthStore = defineStore("auth", () => {
  const user = ref<LightNovelUser | null>(null);
  const restoring = ref(false);
  const refreshing = ref(false);
  const isAuthenticated = computed(() => user.value !== null);

  async function restore() {
    restoring.value = true;
    try {
      user.value = await restoreUser();
      return user.value;
    } finally {
      restoring.value = false;
    }
  }

  async function login(email: string, password: string) {
    user.value = await loginRequest(email, password);
    return user.value;
  }

  async function register(userName: string, email: string, password: string, code: string, inviteCode = "") {
    user.value = await registerRequest(userName, email, password, code, inviteCode);
    return user.value;
  }

  async function refreshProfile() {
    refreshing.value = true;
    try {
      user.value = await restoreUser();
    } finally {
      refreshing.value = false;
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
    refreshing,
    isAuthenticated,
    restore,
    login,
    register,
    refreshProfile,
    logout,
    expire,
  };
});
