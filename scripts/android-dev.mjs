import { spawn, spawnSync } from "node:child_process";
import { existsSync, openSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const AVD_NAME = process.env.MOVEL_AVD ?? "movel_api36";
const BOOT_TIMEOUT_MS = 5 * 60 * 1000;

const sdkRoot = process.env.ANDROID_HOME ?? process.env.ANDROID_SDK_ROOT;
if (!sdkRoot) {
  console.error("未设置 ANDROID_HOME 或 ANDROID_SDK_ROOT");
  process.exit(1);
}

const adb = join(sdkRoot, "platform-tools", "adb");
const emulator = join(sdkRoot, "emulator", "emulator");
for (const binary of [adb, emulator]) {
  if (!existsSync(binary)) {
    console.error(`找不到 ${binary}`);
    process.exit(1);
  }
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

function readyDevices() {
  const { stdout } = spawnSync(adb, ["devices"], { encoding: "utf8" });
  return stdout
    .split("\n")
    .map((line) => line.trim().split(/\s+/))
    .filter((parts) => parts.length === 2 && parts[1] === "device")
    .map(([serial]) => serial);
}

function bootCompleted(serial) {
  const { stdout } = spawnSync(
    adb,
    ["-s", serial, "shell", "getprop", "sys.boot_completed"],
    { encoding: "utf8" },
  );
  return stdout.trim() === "1";
}

let serial = readyDevices()[0];

if (serial) {
  console.log(`复用已连接设备 ${serial}`);
} else {
  console.log(`未检测到设备，启动模拟器 ${AVD_NAME} ...`);
  // 保留 emulator 输出：它失败时（缺显示环境、GPU 不可用）只在 stderr 说明原因。
  const logPath = join(tmpdir(), "movel-emulator.log");
  const logFd = openSync(logPath, "w");
  spawn(emulator, ["-avd", AVD_NAME, "-gpu", "host"], {
    detached: true,
    stdio: ["ignore", logFd, logFd],
  }).unref();

  const deadline = Date.now() + BOOT_TIMEOUT_MS;
  while (!serial && Date.now() < deadline) {
    await sleep(2000);
    serial = readyDevices()[0];
  }
  if (!serial) {
    console.error(
      `等待模拟器连接超时（${BOOT_TIMEOUT_MS / 1000}s），模拟器日志: ${logPath}`,
    );
    process.exit(1);
  }

  console.log(`模拟器 ${serial} 已连接，等待开机完成 ...`);
  while (!bootCompleted(serial) && Date.now() < deadline) {
    await sleep(2000);
  }
  if (!bootCompleted(serial)) {
    console.error(`等待开机完成超时（${BOOT_TIMEOUT_MS / 1000}s）`);
    process.exit(1);
  }
  console.log("开机完成，开始构建。");
}

// `tauri android dev` 不接受 --target：它从连接的设备自动推断 target
// （日志中的 "Detected connected device ... with target"）。
const child = spawn(
  "pnpm",
  ["tauri", "android", "dev", ...process.argv.slice(2)],
  { stdio: "inherit" },
);
child.on("exit", (code) => process.exit(code ?? 0));
