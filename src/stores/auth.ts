import { computed, ref } from "vue";
import { defineStore } from "pinia";
import {
  login as loginRequest,
  logout as logoutRequest,
  register as registerRequest,
  restoreUser,
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
      return user.value;
    } finally {
      restoring.value = false;
    }
  }

  async function login(input: LoginInput) {
    user.value = await loginRequest(input);
    return user.value;
  }

  async function register(input: RegisterInput) {
    user.value = await registerRequest(input);
    return user.value;
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
    logout,
    expire,
  };
});
