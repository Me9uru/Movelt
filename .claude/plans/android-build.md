# 打包 Android Debug APK 方案

## 目标
把当前 Tauri 2 + Vue 3 小说阅读器打包成 Android **Debug APK**,只构建不安装。
用户选择:JDK 用 Android Studio 自带 JBR 25(失败再换 JDK 21)。

## 现状(已查清)
- 已有:Tauri CLI 2.11.4、Rust 1.97.1、Android SDK(platforms android-37、build-tools 36.0.0、platform-tools/adb、emulator)、Android Studio `/opt/android-studio`(自带 JBR / JDK 25)、reqwest 用 rustls-tls(Android 兼容)。
- 缺失:NDK、sdkmanager/cmdline-tools、Rust Android targets、环境变量(ANDROID_HOME/NDK_HOME/JAVA_HOME)、Tauri Android 工程(`src-tauri/gen/android/`)。
- 系统无 `java`/`sdkmanager` 命令;Arch 官方源无 android-ndk/cmdline-tools 包。

## 步骤

### 1. 安装 Rust Android targets
```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

### 2. 安装 cmdline-tools + NDK(无需 sudo,装到 ~/Android/Sdk)
- 下载 Google 官方 commandlinetools-linux zip 到临时目录并解压到 `~/Android/Sdk/cmdline-tools/latest/`。
- 接受许可:`sdkmanager --licenses`。
- 安装 NDK 27(Tauri 2 推荐):`sdkmanager --install "ndk;27.2.12479018"`(若该版本号不存在,先 `sdkmanager --list` 取最新 27.x)。

### 3. 配置环境变量
本次会话先 export,并追加到 `~/.bashrc` 以便日后复用:
```bash
export JAVA_HOME=/opt/android-studio/jbr
export ANDROID_HOME=$HOME/Android/Sdk
export ANDROID_SDK_ROOT=$ANDROID_HOME
export NDK_HOME=$ANDROID_HOME/ndk/<installed-version>   # 装完后回填实际版本目录
export PATH=$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/cmdline-tools/latest/bin:$PATH
```

### 4. 初始化 Tauri Android 工程
```bash
pnpm tauri android init
```
生成 `src-tauri/gen/android/`(Gradle 工程 + AndroidManifest.xml)。

### 5. 配置适配检查(只读核对,按需调整)
- **网络权限**:Tauri Android 模板默认含 `INTERNET` 权限;API base 是 https,无需 cleartext。核对 `AndroidManifest.xml`。
- **tauri-plugin-dev-invoke**:这是 debug-only 的 localhost:3030 桥接。核对它是否在 Android target 下编译通过;若报错,用 Cargo target-specific 依赖在 Android 下排除(不改动桌面行为)。
- **identifier** `com.meguru.novel` 合法,无需改。

### 6. 构建 Debug APK
```bash
pnpm tauri android build --debug
```
产物路径:`src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk`

## 风险与备选
1. **JDK 25 + Gradle**:Tauri Android 模板的 Gradle 可能不支持 JDK 25,典型报错 `Unsupported class file major version`。若出现,装 `jdk21-openjdk`(需 `sudo pacman -S jdk21-openjdk`)并把 `JAVA_HOME` 切到 `/usr/lib/jvm/java-21-openjdk`。JDK 21 对 Gradle 8/9 都兼容。
2. **NDK 版本不匹配**:Tauri CLI 可能在 init 时要求特定 NDK 版本,按提示用 sdkmanager 装对应版本。
3. **dev-invoke 插件**:若 Android 下编译/运行异常,按步骤 5 的方式排除。
4. **下载体积**:cmdline-tools(~150MB)+ NDK 27(~1.3GB)+ Rust targets(几百 MB),首次构建还会拉 Gradle/AGP 依赖,耗时较长。

## 不做的事
- 不安装到设备/模拟器(用户选"只构建")。
- 不做 release 签名。
- 不改动桌面端构建行为。
