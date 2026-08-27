<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, watch } from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import MainNavigation from "./layout/MainNavigation.vue";
import { useAuthStore } from "./stores/auth";
import type { AppRouteName, LibraryRouteName } from "./types/router";
import { showError } from "./utils/error";

const route = useRoute();
const router = useRouter();
const auth = useAuthStore();
const view = computed<AppRouteName>(() => {
  const routeName = route.name;
  return routeName === "bookshelf" ||
    routeName === "detail" ||
    routeName === "reader" ||
    routeName === "comic" ||
    routeName === "comic-detail" ||
    routeName === "comic-reader" ||
    routeName === "settings" ||
    routeName === "login"
    ? routeName
    : "novels";
});

function redirectToLogin() {
  if (route.name === "login") return;
  void router.replace({ name: "login", query: { redirect: route.fullPath } });
}

function handleAuthenticationExpired() {
  auth.expire();
  redirectToLogin();
}

function handleAndroidBack(event: Event) {
  if (
    view.value !== "detail" &&
    view.value !== "reader" &&
    view.value !== "comic-detail"
  )
    return;
  event.preventDefault();
  if (window.history.state?.back) router.back();
  else if (view.value === "reader" && typeof route.params.bookId === "string") {
    void router.replace({
      name: "detail",
      params: { bookId: route.params.bookId },
      query: route.query,
    });
  } else
    void router.replace({
      name: view.value === "comic-detail" ? "comic" : "novels",
    });
}

function navigate(nextView: LibraryRouteName) {
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
    <RouterView v-slot="{ Component }">
      <component :is="Component" v-if="view === 'login' || auth.user" />
    </RouterView>
    <MainNavigation v-if="auth.user" :view="view" @navigate="navigate" />
  </div>
</template>
