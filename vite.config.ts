import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import Components from "unplugin-vue-components/vite";
import { ElementPlusResolver } from "unplugin-vue-components/resolvers";
// @ts-expect-error type error without @types/node package
import process from "node:process";
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [
    vue(),
    Components({ resolvers: [ElementPlusResolver()], dts: "src/components.d.ts" }),
  ],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || "0.0.0.0",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
    // Keep the debug invoke bridge bound to the host's loopback interface while
    // allowing browsers on the LAN to reach it through the Vite dev server.
    proxy: {
      "/__tauri_invoke": {
        target: "http://127.0.0.1:3030",
        rewrite: () => "/",
      },
    },
  },
  preview: {
    host: "0.0.0.0",
  },
}));
