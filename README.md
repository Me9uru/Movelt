<div align="center">
  <img src="src-tauri/icons/icon.png" width="112" alt="Movel Logo" />

# Movel

**LightNovel 轻书架的非官方第三方客户端**

基于 Tauri 2、Vue 3 与 Rust 构建，提供小说与漫画浏览、书架、阅读、账号与进度同步能力。

[![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![Vue.js](https://img.shields.io/badge/Vue.js-3-42B883?logo=vuedotjs&logoColor=white)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-2021-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-6-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

</div>

## 关于 Movel

Movel 是 [LightNovel 轻书架](https://www.lightnovel.life/) 的非官方第三方客户端。前端提供轻量、响应式的阅读界面；Rust 后端作为官方 API 的客户端边界，负责认证、会话续期、请求与响应校验及领域 DTO 映射；两端通过 Tauri IPC 通信。

> [!WARNING]
> 项目目前处于快速迭代阶段。功能、界面与数据契约可能随版本调整；欢迎通过 Issue 反馈问题和建议。

## 界面预览

![Movel 小说首页：推荐书单、热门作品与底部导航](docs/images/novels-overview.png)

## 功能

- **小说与漫画发现**：首页书单、排行榜、关键词搜索与搜索结果分页；漫画首页会优先呈现最近更新。
- **作品详情与目录**：查看封面、作者、状态、简介、标签、分卷与章节，并可直接加入或移出书架。
- **账号与官方书架**：支持登录、邮箱验证码注册、恢复登录状态；小说与漫画收藏、阅读位置均与官方服务同步。
- **小说阅读器**：连续滚动或分页阅读，宽屏分页自动采用单页/双页布局；支持键盘、触摸与按钮翻页。
- **漫画阅读器**：滚动或分页观看，按批次加载图片、自动记录页码，并可在章节间连续阅读。
- **可定制阅读体验**：纸张、明亮、夜间三套主题；小说还可设置字体、简繁转换、字号、行距、字距、段距和正文宽度。
- **更顺畅的续读**：原生侧缓存章节内容，并在后台预加载相邻章节，减少翻章等待。
- **响应式与移动端行为**：适配桌面与窄屏窗口，并处理 Android 返回导航。

## 特点

| 特点 | 说明 |
| --- | --- |
| 轻量原生 | Tauri 使用系统 WebView，应用不打包完整浏览器内核。 |
| Rust 服务边界 | 官方 API、SignalR、认证、令牌刷新、压缩响应与 DTO 映射均由 Rust 处理。 |
| 凭据不出原生层 | 刷新凭据仅写入系统凭据库，前端不会接收或保存访问令牌。 |
| 官方服务同步 | 认证、书架、阅读位置与内容由 LightNovel 轻书架服务维护。 |
| 阅读优先 | 提供多主题、响应式排版、丰富排版参数和相邻章节预加载。 |
| 开发友好 | Debug 模式提供只监听 localhost 的浏览器调用桥，方便使用 Chrome DevTools 调试。 |

## 技术栈

### 前端

- [Vue 3](https://vuejs.org/) + Composition API
- [TypeScript](https://www.typescriptlang.org/)
- [Varlet UI](https://www.varletjs.com/)
- [Vite](https://vite.dev/)

### 原生后端

- [Tauri 2](https://tauri.app/)：窗口、IPC 与跨平台应用打包
- [Rust](https://www.rust-lang.org/) + Tokio：业务逻辑与异步任务
- [Reqwest](https://docs.rs/reqwest/)：HTTP 请求与压缩传输
- [tokio-tungstenite](https://docs.rs/tokio-tungstenite/)：SignalR WebSocket 通信
- Keyring：系统凭据库中的刷新凭据存储（通过本地 Tauri 插件）
- Serde / Thiserror：数据序列化与结构化错误处理

## 快速开始

### 环境要求

- Node.js 与 [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install/)
- 当前平台所需的 [Tauri 系统依赖](https://v2.tauri.app/start/prerequisites/)

### 启动完整应用

```bash
pnpm install
pnpm tauri dev
```

只开发前端界面时，可以运行：

```bash
pnpm dev
```

### 构建

```bash
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 开发与检查

```bash
# 前端类型检查与生产构建
pnpm build

# Rust 测试
cd src-tauri && cargo test

# Rust 格式与静态检查
cd src-tauri && cargo fmt --check
cd src-tauri && cargo clippy --all-targets -- -D warnings
```

## 在 Chrome 中调试

Movel 在 Debug 构建中提供 HTTP invoke bridge。保持完整开发进程运行：

```bash
pnpm tauri dev
```

随后在 Chrome 中打开 [http://localhost:1420](http://localhost:1420)。前端发起的 `invoke()` 调用会经由 `http://127.0.0.1:3030` 转发到 Tauri command handler，因此可以在 DevTools 的 Network 面板中观察请求；原生窗口仍然使用正常的 IPC 通道。

> [!IMPORTANT]
> 调试桥仅在 Rust Debug 构建中启动，并且只监听 `127.0.0.1`。它允许浏览器来源通过 CORS，请勿将其暴露为生产 API。如果 `3030` 端口已被占用，请先关闭冲突进程。

## 项目结构

```text
Movel/
├── src/
│   ├── components/       # 通用、作品、布局与阅读器组件
│   ├── composables/      # 发现页、书架与阅读设置逻辑
│   ├── domain/           # 面向界面的类型化领域模型
│   ├── pages/            # 认证、小说、漫画、书架与设置页面
│   ├── services/         # 类型化 Tauri command 调用封装
│   ├── stores/           # 登录状态与本地阅读设置
│   └── styles/           # 主题、布局、作品与阅读器样式
├── src-tauri/
│   ├── capabilities/     # 最小化的 Tauri 权限配置
│   └── src/
│       ├── api/          # 官方 API、认证、HTTP 与 SignalR 连接
│       ├── commands/     # 用户、小说、漫画与书架 commands
│       ├── dto/          # 上游与命令 DTO
│       └── reader_cache.rs # 章节与漫画页面的 LRU 内存缓存
├── docs/images/          # README 展示图片
└── scripts/              # 开发与平台同步脚本
```

## 声明

Movel 是独立开发的非官方客户端，与 LightNovel 轻书架及其运营方没有隶属或授权关系。作品内容、封面与相关信息均由服务提供方提供，其版权归原作者及权利人所有。请遵守当地法律法规与服务使用条款，支持正版阅读。

## 开源协议

本项目基于 [MIT License](LICENSE) 开源。
