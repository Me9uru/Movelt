import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
import "./styles/theme.css";
import "./styles/app.css";
import "./styles/reader.css";

// Element Plus 组件与样式通过 resolver 在编译期按需引入，无需 app.use 与全量 CSS。
createApp(App).use(router).mount("#app");
