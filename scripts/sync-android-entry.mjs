import { copyFileSync, existsSync, mkdirSync } from "node:fs";
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
  copyFileSync(
    new URL("../src-tauri/mobile/android/NativeCrashTrace.kt", import.meta.url),
    new URL("../src-tauri/gen/android/app/src/main/java/com/meguru/movel/NativeCrashTrace.kt", import.meta.url),
  );
  const testTarget = new URL("../src-tauri/gen/android/app/src/test/java/com/meguru/movel/NativeCrashTraceTest.kt", import.meta.url);
  mkdirSync(dirname(fileURLToPath(testTarget)), { recursive: true });
  copyFileSync(new URL("../src-tauri/mobile/android/tests/NativeCrashTraceTest.kt", import.meta.url), testTarget);
}
