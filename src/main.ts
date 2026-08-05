import { createApp } from "vue";
import App from "./App.vue";
import { setupDevInvoke } from "./dev-invoke";

// In an external browser, route @tauri-apps/api invoke calls through the
// debug-only HTTP bridge started by the Rust plugin. Inside Tauri this is a
// no-op because __TAURI_INTERNALS__ already exists.
if (import.meta.env.DEV) setupDevInvoke();

// Element Plus 组件与样式按需引入(unplugin-vue-components + ElementPlusResolver),
// 组件及 v-loading 指令均由 resolver 在编译期自动注入,无需 app.use 与全量 CSS。
createApp(App).mount("#app");
