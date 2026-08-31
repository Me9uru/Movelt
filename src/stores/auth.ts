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
import { useBookshelfStore } from "./bookshelf";
import { useDiscoverySearchStore } from "./discoverySearch";

export const useAuthStore = defineStore("auth", () => {
  const bookshelf = useBookshelfStore();
  const discoverySearch = useDiscoverySearchStore();
  const user = ref<LightNovelUser | null>(null);
  // 首次渲染就展示启动占位，直到应用完成持久化登录态恢复。
  const restoring = ref(true);
  const isAuthenticated = computed(() => user.value !== null);

  const restore = async () => {
    restoring.value = true;
    try {
      user.value = await restoreUser();
      void signInIfNeeded();
      return user.value;
    } finally {
      restoring.value = false;
    }
  }

  const login = async (input: LoginInput) => {
    user.value = await loginRequest(input);
    bookshelf.clear();
    discoverySearch.clear();
    void signInIfNeeded();
    return user.value;
  }

  const register = async (input: RegisterInput) => {
    user.value = await registerRequest(input);
    bookshelf.clear();
    discoverySearch.clear();
    void signInIfNeeded();
    return user.value;
  }

  const signInIfNeeded = async () => {
    if (user.value?.Growth?.TodaySigned !== false) return false;
    try {
      user.value = await signInRequest();
      return true;
    } catch {
      // 自动签到不应妨碍登录或恢复会话；下次启动时会再次尝试。
      return false;
    }
  }

  const logout = async () => {
    await logoutRequest();
    user.value = null;
    bookshelf.clear();
    discoverySearch.clear();
  }

  const expire = () => {
    user.value = null;
    bookshelf.clear();
    discoverySearch.clear();
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
