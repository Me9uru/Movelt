import { createApp } from "vue";
import App from "./App.vue";
import { setupDevInvoke } from "./dev-invoke";
import { router } from "./router";
import "./styles/theme.css";
import "./styles/app.css";
import "./styles/reader.css";

// In an external browser, route @tauri-apps/api invoke calls through the
// debug-only HTTP bridge started by the Rust plugin. Inside Tauri this is a
// no-op because __TAURI_INTERNALS__ already exists.
if (import.meta.env.DEV) setupDevInvoke();

// Element Plus 组件与样式通过 resolver 在编译期按需引入，无需 app.use 与全量 CSS。
createApp(App).use(router).mount("#app");
