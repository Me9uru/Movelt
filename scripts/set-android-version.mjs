import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const nextVersion = process.argv[2];
const dryRun = process.argv.includes("--dry-run");
const semverPattern = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;

if (!nextVersion || !semverPattern.test(nextVersion)) {
  console.error("用法: pnpm version:android <major.minor.patch>");
  process.exit(1);
}

const paths = {
  packageJson: fileURLToPath(new URL("../package.json", import.meta.url)),
  tauriConfig: fileURLToPath(
    new URL("../src-tauri/tauri.conf.json", import.meta.url),
  ),
  cargoToml: fileURLToPath(
    new URL("../src-tauri/Cargo.toml", import.meta.url),
  ),
  cargoLock: fileURLToPath(
    new URL("../src-tauri/Cargo.lock", import.meta.url),
  ),
};

const packageJsonText = readFileSync(paths.packageJson, "utf8");
const tauriConfigText = readFileSync(paths.tauriConfig, "utf8");
const packageJson = JSON.parse(packageJsonText);
const tauriConfig = JSON.parse(tauriConfigText);
const currentVersion = tauriConfig.version;
const currentVersionCode = tauriConfig.bundle?.android?.versionCode;

function compareVersions(left, right) {
  const leftParts = left.split(".").map(Number);
  const rightParts = right.split(".").map(Number);
  for (let index = 0; index < 3; index += 1) {
    if (leftParts[index] !== rightParts[index])
      return leftParts[index] - rightParts[index];
  }
  return 0;
}

if (typeof currentVersion !== "string" || !semverPattern.test(currentVersion)) {
  console.error("tauri.conf.json 中的当前版本不是 major.minor.patch 格式");
  process.exit(1);
}
if (packageJson.version !== currentVersion) {
  console.error("package.json 与 tauri.conf.json 的当前版本不一致");
  process.exit(1);
}
if (compareVersions(nextVersion, currentVersion) <= 0) {
  console.error(`新版本 ${nextVersion} 必须高于当前版本 ${currentVersion}`);
  process.exit(1);
}
if (!Number.isInteger(currentVersionCode) || currentVersionCode < 1) {
  console.error("tauri.conf.json 缺少有效的 bundle.android.versionCode");
  process.exit(1);
}

const nextVersionCode = currentVersionCode + 1;
if (nextVersionCode > 2_100_000_000) {
  console.error("Android versionCode 已超过允许的最大值 2100000000");
  process.exit(1);
}

function replaceRequired(content, pattern, replacement, fileName) {
  const updated = content.replace(pattern, replacement);
  if (updated === content) {
    console.error(`无法更新 ${fileName}，请检查版本字段格式`);
    process.exit(1);
  }
  return updated;
}

const nextPackageJson = replaceRequired(
  packageJsonText,
  /(^  "version": ")[^"]+(",$)/m,
  `$1${nextVersion}$2`,
  "package.json",
);
const nextTauriConfigVersion = replaceRequired(
  tauriConfigText,
  /(^  "version": ")[^"]+(",$)/m,
  `$1${nextVersion}$2`,
  "tauri.conf.json version",
);
const nextTauriConfig = replaceRequired(
  nextTauriConfigVersion,
  /(^      "versionCode": )\d+(,?$)/m,
  `$1${nextVersionCode}$2`,
  "tauri.conf.json versionCode",
);
const cargoToml = replaceRequired(
  readFileSync(paths.cargoToml, "utf8"),
  /(^\[package\][\s\S]*?^version = ")[^"]+("$)/m,
  `$1${nextVersion}$2`,
  "Cargo.toml",
);
const cargoLock = replaceRequired(
  readFileSync(paths.cargoLock, "utf8"),
  /(^\[\[package\]\]\nname = "movel"\nversion = ")[^"]+("$)/m,
  `$1${nextVersion}$2`,
  "Cargo.lock",
);

if (!dryRun) {
  writeFileSync(paths.packageJson, nextPackageJson);
  writeFileSync(paths.tauriConfig, nextTauriConfig);
  writeFileSync(paths.cargoToml, cargoToml);
  writeFileSync(paths.cargoLock, cargoLock);
}

console.log(
  `Android 发布版本${dryRun ? "将更新" : "已更新"}: ${currentVersion} (${currentVersionCode}) -> ${nextVersion} (${nextVersionCode})`,
);
