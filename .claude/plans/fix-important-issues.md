# 修复计划:4 项 🟡 重要问题

## #1 Android 改名一致性

**问题:** `tauri.conf.json` identifier 已改为 `com.meguru.movel`,但 `MainActivity.kt` 包名与 sync 脚本目标路径仍是 `com.meguru.novel`。重新 `tauri android init` 会导致 sync 脚本静默跳过,丢失返回键处理。

**改动:**
- `src-tauri/mobile/android/MainActivity.kt:1`:`package com.meguru.novel` → `package com.meguru.movel`
- `scripts/sync-android-entry.mjs:10`:目标路径 `com/meguru/novel/` → `com/meguru/movel/`

**说明:** `src-tauri/gen/` 是 gitignored 生成产物(`src-tauri/.gitignore:9`),不在版本控制,本次不动。改完源码后需重新运行 `tauri android init` 以按新 identifier 重新生成脚手架(否则旧 gen 仍是 `com.meguru.novel`,sync 脚本目标目录不存在会静默跳过)。此步骤需用户在有 Android SDK 环境时执行,不在本次代码改动内。

## #2 WUBA_BASE_URL 透传到 parser

**问题:** `client.rs` 支持 `WUBA_BASE_URL` 环境变量覆盖,但 `parser.rs` 硬编码 `m.5859ycdh.com` 校验 host,导致换主站后链接/封面被解析器丢弃。

**改动:**
- `src-tauri/src/novel/wuba/client.rs`:新增 `pub(super) fn base_url(&self) -> &Url { &self.base_url }`
- `src-tauri/src/novel/wuba/parser.rs`:
  - 删除 `const BASE_URL`
  - `search(html, base_url: &Url)`、`overview(html, novel_id, base_url: &Url)` 增加 `base_url` 参数
  - `chapter` 签名不变(不解析链接,无需 host)
  - `source_path(value, base_url)`、`image_url(element, base_url)` 增加参数,用传入的 `base_url` 替代 `Url::parse(BASE_URL)`,host 校验改为 `url.host_str() == base_url.host_str()`
  - 更新两个测试(`parses_search_result_and_lazy_cover`、`parses_detail_and_catalogue_from_one_page`)传入 `&Url::parse("http://m.5859ycdh.com/").unwrap()`;`detects_and_merges_chapter_pages` 不受影响
- `src-tauri/src/novel/wuba/mod.rs`:`parser::search(&html, self.client.base_url())`、`parser::overview(&html, novel_id, self.client.base_url())`

## #3 引入 async-trait,用 Arc<dyn NovelSource>

**问题:** `NovelSource` trait 定义了却未被多态使用,`Provider` 枚举手写转发,加源需改 5 处 match。

**改动:**
- `src-tauri/Cargo.toml`:依赖加 `async-trait = "0.1"`
- `src-tauri/src/novel/provider.rs`:trait 加 `#[async_trait]`,删除 `#[allow(async_fn_in_trait)]`
- `src-tauri/src/novel/bilinovel/mod.rs`:`impl NovelSource for BilinovelSource` 块上加 `#[async_trait]`
- `src-tauri/src/novel/wuba/mod.rs`:`impl NovelSource for WubaSource` 块上加 `#[async_trait]`
- `src-tauri/src/novel/mod.rs`:
  - 删除 `Provider` enum 及其全部 `impl`(约 50 行)
  - `NovelState.providers: Vec<Arc<dyn NovelSource>>`
  - `new()`:`vec![Arc::new(WubaSource::new()?), Arc::new(BilinovelSource::new()?)]`
  - `provider(source)` 返回 `Arc<dyn NovelSource>`(逻辑不变)
  - `list_novel_sources` / `search_novels` / `get_novel_overview` / `prefetch_chapters` 直接调用 `provider.search()` 等,删除 enum 转发层
  - `state.chapter` 内 spawn 的任务通过 `Arc::clone(&provider)` 捕获(已是 `Arc<dyn NovelSource>`,方法不变)
  - 测试 `rejects_unknown_source`、`chapter_cache_evicts...` 不受影响

## #4 返回键可导航,结果按需丢弃

**问题:** 加载态最长 60s 锁死所有导航(返回键、返回按钮、Android 返回均被拦截),无逃逸途径。

**改动(`src/App.vue`):**
- 新增模块级 `let activeLoadSeq = 0;`
- `run()`:
  - 保留开头 `if (loading.value) return null;`(防重复触发)
  - 开头 `const seq = ++activeLoadSeq;`
  - `try`/`catch` 完成后 `if (seq !== activeLoadSeq) return null;`(过期则丢弃结果)
  - `finally` 仅在 `seq === activeLoadSeq` 时清 `loading`/`loadingAction`
- `back()`:去掉 `if (loading.value) return;`;若 `loading.value` 为 true,先 `activeLoadSeq++` + 清 `loading`/`loadingAction`(使在途加载结果失效、立即隐藏遮罩),再 `window.history.back()`
- `handleAndroidBack`:去掉 `if (loading.value) { event.preventDefault(); return; }`,允许加载时返回
- 模板:返回按钮(`back-button`)的 `:disabled="loading"` 删除;其他操作按钮(搜索、加入书架、继续阅读、章节列表项)保留 `:disabled="loading"`(防重复触发)
- 后台请求继续跑完,结果进后端缓存(对 chapter 命中缓存;overview 无前端缓存但请求已发出,结果被前端丢弃)

## 验证步骤
1. `cd src-tauri && cargo check --all-targets` — 编译通过
2. `cd src-tauri && cargo test` — 10 个测试仍通过(1 个 ignored)
3. `npx vue-tsc --noEmit` — 类型检查通过
4. 手动(用户):加载中按返回键/返回按钮,验证可立即返回且不报错;后台请求完成后不强制跳回

## 不在本次范围
- 🟢 小问题(#5-#10)、💡 建议(#11-#12)留作后续
- `tauri android init` 重新生成脚手架(需用户 Android 环境)
