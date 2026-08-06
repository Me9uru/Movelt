import { copyFileSync, existsSync } from "node:fs";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";

const source = fileURLToPath(
  new URL("../src-tauri/mobile/android/MainActivity.kt", import.meta.url),
);
const target = fileURLToPath(
  new URL(
    "../src-tauri/gen/android/app/src/main/java/com/meguru/movel/MainActivity.kt",
    import.meta.url,
  ),
);

if (existsSync(dirname(target))) {
  copyFileSync(source, target);
}
