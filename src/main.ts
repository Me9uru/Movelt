import { createApp } from "vue";
import { createPinia } from "pinia";
import { StyleProvider, Themes } from "@varlet/ui";
import "@varlet/ui/es/style.mjs";
import "@varlet/touch-emulator";
import App from "./App.vue";
import { setupDevInvoke } from "./dev-invoke";
import { router } from "./router";
import { showError, showErrorDialog } from "./utils/error";
import "./styles/theme.css";
import "./styles/app.css";
import "./styles/components.css";
import "./styles/novel.css";
import "./styles/manga.css";
import "./styles/responsive.css";
import "./styles/reader-content.css";
import "./styles/reader.css";

if (import.meta.env.DEV) setupDevInvoke();

// Varlet controls are auto-imported on demand. The MD3 palette follows the app theme.
StyleProvider(Themes.md3Light);
const app = createApp(App);

app.config.errorHandler = (error, _instance, info) => {
  console.error(error);
  showError(error, `页面发生异常（${info}）`);
};

window.addEventListener("unhandledrejection", (event) => {
  event.preventDefault();
  showErrorDialog(event.reason);
});

router.onError((error) => {
  showErrorDialog(error, "无法打开页面，请刷新后重试");
});

app.use(createPinia()).use(router).mount("#app");
