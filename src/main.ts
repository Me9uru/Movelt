import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { setupDevInvoke } from "./dev-invoke";
import { router } from "./router";
import { showError } from "./utils/error";
import "./styles/theme.css";
import "./styles/app.css";
import "./styles/reader.css";

if (import.meta.env.DEV) setupDevInvoke();

// Element Plus 组件与样式通过 resolver 在编译期按需引入，无需 app.use 与全量 CSS。
const app = createApp(App);

app.config.errorHandler = (error, _instance, info) => {
  console.error(error);
  showError(error, `页面发生异常（${info}）`);
};

window.addEventListener("unhandledrejection", (event) => {
  event.preventDefault();
  showError(event.reason);
});

app.use(createPinia()).use(router).mount("#app");
