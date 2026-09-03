<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, watch } from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import AppStartupScreen from "./components/app/AppStartupScreen.vue";
import MainNavigation from "./components/app/MainNavigation.vue";
import { useAuthStore } from "./stores/auth";
import type { AppRouteName, NavigationRouteName } from "./types/router";
import { showError } from "./utils/error";

const route = useRoute();
const router = useRouter();
const auth = useAuthStore();
const view = computed<AppRouteName>(() => {
  const routeName = route.name;
  return routeName === "bookshelf" ||
    routeName === "novel-detail" ||
    routeName === "novel-reader" ||
    routeName === "comic" ||
    routeName === "comic-detail" ||
    routeName === "comic-reader" ||
    routeName === "login"
    ? routeName
    : "novels";
});

const redirectToLogin = () => {
  if (route.name === "login") return;
  void router.replace({ name: "login", query: { redirect: route.fullPath } });
}

const handleAuthenticationExpired = () => {
  auth.expire();
  redirectToLogin();
}

const handleAndroidBack = (event: Event) => {
  if (
    view.value !== "novel-detail" &&
    view.value !== "novel-reader" &&
    view.value !== "comic-detail" &&
    view.value !== "comic-reader"
  )
    return;
  event.preventDefault();
  if (window.history.state?.back) router.back();
  else if (
    view.value === "novel-reader" &&
    typeof route.params.bookId === "string"
  ) {
    void router.replace({
      name: "novel-detail",
      params: { bookId: route.params.bookId },
      query: route.query,
    });
  } else
    void router.replace({
      name:
        view.value === "comic-detail" || view.value === "comic-reader"
          ? "comic"
          : "novels",
    });
}

const navigate = (nextView: NavigationRouteName) => {
  void router.replace({ name: nextView });
}

watch(
  () => auth.user,
  (user, previousUser) => {
    if (!user && previousUser) redirectToLogin();
  },
);

onMounted(() => {
  window.addEventListener("movel:android-back", handleAndroidBack);
  window.addEventListener(
    "movel:authentication-expired",
    handleAuthenticationExpired,
  );
  void auth
    .restore()
    .then((user) => {
      if (!user) redirectToLogin();
    })
    .catch((error: unknown) => {
      redirectToLogin();
      showError(error);
    });
});

onBeforeUnmount(() => {
  window.removeEventListener("movel:android-back", handleAndroidBack);
  window.removeEventListener(
    "movel:authentication-expired",
    handleAuthenticationExpired,
  );
});
</script>

<template>
  <div class="page-bg">
    <AppStartupScreen v-if="auth.restoring" />
    <template v-else>
      <RouterView v-slot="{ Component }">
        <component :is="Component" v-if="view === 'login' || auth.user" />
      </RouterView>
      <MainNavigation v-if="auth.user" :view="view" @navigate="navigate" />
    </template>
  </div>
</template>
