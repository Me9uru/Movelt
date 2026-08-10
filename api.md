# pywenku8api HTTP API

本文档描述 `pywenku8api` 当前对外提供的 FastAPI HTTP 接口，供 Movel 等小说客户端接入。

## 1. 服务概览

- 默认地址：`http://127.0.0.1:8000`
- Swagger UI：`GET /docs`
- OpenAPI：`GET /openapi.json`
- API 版本：`0.1.0`
- 字符编码：JSON 与正文均为 UTF-8
- 请求方法：当前所有业务接口均为 `GET`

服务默认只监听本机回环地址。如需允许局域网访问，启动前设置：

```bash
export WENKU8_HOST=0.0.0.0
```

## 2. 登录与鉴权模型

本服务没有 `/login` 接口，也不要求客户端传 Token、Cookie 或 Authorization 请求头。

服务启动时读取 `.env` 中的 `WENKU8_USERNAME` 和 `WENKU8_PASSWORD`，登录 Wenku8 后在服务进程内维护一份共享会话。所有客户端请求共用该登录态。

客户端应首先调用 `GET /health`：

```json
{
  "logged_in": true
}
```

- `logged_in=true`：全部接口可以正常调用。
- `logged_in=false`：仅健康检查、封面、整本 TXT 和图片透传接口可用；其余接口返回 HTTP 401。

无需登录的接口：

- `GET /health`
- `GET /novel/cover/{aid}`
- `GET /novel/full/{aid}`
- `GET /picture`

## 3. 通用参数

### `lang`

多数接口接受可选查询参数 `lang`：

| 值 | 含义 |
| --- | --- |
| `zh_CN` | 简体中文，默认值 |
| `zh_TW` | 繁体中文 |

内部枚举值 `gbk`、`big5` 也可被当前实现识别，但客户端应优先使用公开值 `zh_CN`、`zh_TW`。

### 分页参数 `page`

- 类型：整数
- 默认值：`1`
- 最小值：`1`

### 标识符

| 名称 | 类型 | 含义 |
| --- | --- | --- |
| `aid` | integer | 小说/文章 ID |
| `cid` | integer | 章节 ID |
| `vid` | integer | 分卷 ID |
| `uid` | integer | 用户 ID |
| `rid` | integer | 书评 ID |
| `bid` | integer | 书架条目或书架分类 ID，具体含义随接口而定 |

## 4. 接口一览

| 分类 | 接口 | 需要登录 | 响应 |
| --- | --- | --- | --- |
| 状态 | `GET /health` | 否 | JSON |
| 小说 | `GET /novel/cover/{aid}` | 否 | JPEG |
| 小说 | `GET /novel/info/{aid}` | 是 | `NovelInfo` |
| 小说 | `GET /novel/index/{aid}` | 是 | `NovelIndex` |
| 小说 | `GET /novel/content/{aid}/{cid}` | 是 | UTF-8 文本 |
| 小说 | `GET /novel/full/{aid}` | 否 | UTF-8 文本 |
| 小说 | `GET /novel/content_via_full/{aid}/{cid}` | 是 | UTF-8 文本 |
| 搜索 | `GET /search` | 是 | `SearchResult` |
| 搜索 | `GET /search/by_name` | 是 | `SearchResult` |
| 搜索 | `GET /search/by_author` | 是 | `SearchResult` |
| 列表 | `GET /novel/list` | 是 | `SearchResult` |
| 列表 | `GET /category` | 是 | `SearchResult` |
| 列表 | `GET /finished` | 是 | `SearchResult` |
| 用户 | `GET /bookshelf` | 是 | `BookshelfItem[]` |
| 用户 | `GET /user/bookshelf/{uid}` | 是 | `NovelCover[]` |
| 评论 | `GET /novel/{aid}/comments` | 是 | `CommentItem[]` |
| 评论 | `GET /comments/{rid}/replies` | 是 | `ReplyItem[]` |
| 用户 | `GET /user/info` | 是 | `UserInfo` |
| 推荐 | `GET /recommend` | 是 | `RecommendBlock[]` |
| 图片 | `GET /picture` | 否 | JPEG |

## 5. 状态接口

### `GET /health`

返回服务当前登录状态。

响应示例：

```json
{
  "logged_in": true
}
```

HTTP 200 仅表示服务进程可响应；调用登录态接口前仍需检查 `logged_in`。

## 6. 小说接口

### `GET /novel/cover/{aid}`

从 Wenku8 图片 CDN 获取小说小尺寸封面。

路径参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `aid` | integer | 是 | 小说 ID |

成功响应：

- HTTP 200
- `Content-Type: image/jpeg`
- 响应体为图片二进制

示例：

```http
GET /novel/cover/1587
```

### `GET /novel/info/{aid}`

获取小说详细信息。

| 参数 | 位置 | 类型 | 必填 | 默认值 |
| --- | --- | --- | --- | --- |
| `aid` | path | integer | 是 | - |
| `lang` | query | string | 否 | `zh_CN` |

成功响应：`NovelInfo`。

```json
{
  "aid": 1587,
  "title": "无职转生～到了异世界就拿出真本事～",
  "author": "理不尽な孙の手",
  "status": "已完结",
  "last_updated": null,
  "intro": "作品简介",
  "tags": ["穿越", "魔法", "冒险"],
  "press": "MF文库J",
  "word_count": null,
  "popularity_level": null,
  "trending_level": null,
  "latest_section": null,
  "copyright": false,
  "animation": true
}
```

`copyright=true` 表示未被版权下架、通常可读；最终仍应以章节接口返回内容为准。

### `GET /novel/index/{aid}`

获取小说目录，目录按分卷组织。

| 参数 | 位置 | 类型 | 必填 | 默认值 |
| --- | --- | --- | --- | --- |
| `aid` | path | integer | 是 | - |
| `lang` | query | string | 否 | `zh_CN` |

成功响应：`NovelIndex`。

```json
{
  "aid": 3765,
  "title": "无职转生～蛇足篇～",
  "author": "理不尽な孙の手",
  "volumes": [
    {
      "vid": 123,
      "title": "第一卷",
      "chapters": [
        {
          "cid": 157137,
          "title": "〈诺伦的婚礼〉"
        }
      ]
    }
  ]
}
```

客户端必须使用目录返回的 `cid` 请求正文，不应自行推算章节 ID。

### `GET /novel/content/{aid}/{cid}`

通过 Wenku8 阅读页获取单章正文。

| 参数 | 位置 | 类型 | 必填 | 默认值 |
| --- | --- | --- | --- | --- |
| `aid` | path | integer | 是 | - |
| `cid` | path | integer | 是 | - |
| `lang` | query | string | 否 | `zh_CN` |

成功响应：

- HTTP 200
- `Content-Type: text/plain; charset=utf-8`
- 响应体为章节正文

正文中的插图使用以下标记：

```text
<!--image-->https://...<!--image-->
```

客户端可以提取 URL 后通过 `GET /picture` 加载图片。

注意：版权下架小说也可能返回 HTTP 200，正文内容为站点的版权提示，例如“因版权问题，文库不再提供该小说的阅读”。客户端不能仅靠状态码判断章节是否可读。

### `GET /novel/full/{aid}`

从 Wenku8 下载 CDN 获取整本 UTF-8 TXT。此接口不要求登录。

| 参数 | 位置 | 类型 | 必填 | 默认值 |
| --- | --- | --- | --- | --- |
| `aid` | path | integer | 是 | - |
| `lang` | query | string | 否 | `zh_CN` |

成功响应为 `text/plain; charset=utf-8`。服务按 `(aid, lang)` 缓存 30 分钟，并在一个下载节点返回 429 时尝试备用节点。

### `GET /novel/content_via_full/{aid}/{cid}`

先读取整本 TXT，再结合目录切分指定章节。适合阅读页暂时不稳定时使用。

| 参数 | 位置 | 类型 | 必填 | 默认值 |
| --- | --- | --- | --- | --- |
| `aid` | path | integer | 是 | - |
| `cid` | path | integer | 是 | - |
| `lang` | query | string | 否 | `zh_CN` |

响应为 UTF-8 纯文本。该接口仍需登录，因为章节切分过程需要获取登录态目录。

## 7. 搜索与列表接口

所有搜索与列表接口均返回 `SearchResult`，并要求服务已经登录。

### `GET /search`

通用搜索接口。

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `keyword` | string | 是 | - | 搜索词，需 URL 编码 |
| `method` | string | 是 | - | `articlename` 或 `author` |
| `page` | integer | 否 | `1` | 页码，最小为 1 |
| `lang` | string | 否 | `zh_CN` | 输出语言 |

```http
GET /search?keyword=%E6%97%A0%E8%81%8C%E8%BD%AC%E7%94%9F&method=articlename&page=1&lang=zh_CN
```

搜索存在 5 秒冷却控制；密集请求会排队等待，而不是返回限流错误。

### `GET /search/by_name`

按书名搜索。参数为 `keyword`、可选 `page` 和可选 `lang`。

```http
GET /search/by_name?keyword=%E6%97%A0%E8%81%8C%E8%BD%AC%E7%94%9F&page=1
```

### `GET /search/by_author`

按作者搜索。参数为 `keyword`、可选 `page` 和可选 `lang`。

```http
GET /search/by_author?keyword=%E7%90%86%E4%B8%8D%E5%B0%BD&page=1
```

### `GET /novel/list`

获取小说排行榜或排序列表。

| 参数 | 类型 | 必填 | 默认值 |
| --- | --- | --- | --- |
| `sort` | `NovelSortMethod` | 是 | - |
| `page` | integer | 否 | `1` |
| `lang` | string | 否 | `zh_CN` |

`sort` 可选值：

| 值 | 含义 |
| --- | --- |
| `allvisit` | 总排行榜 |
| `allvote` | 总推荐榜 |
| `monthvisit` | 月排行榜 |
| `monthvote` | 月推荐榜 |
| `weekvisit` | 周排行榜 |
| `weekvote` | 周推荐榜 |
| `dayvisit` | 日排行榜 |
| `dayvote` | 日推荐榜 |
| `postdate` | 最新入库 |
| `lastupdate` | 最近更新 |
| `goodnum` | 总收藏榜 |
| `size` | 字数排行 |
| `fullflag` | 完结作品 |
| `anime` | 已动画化 |

```http
GET /novel/list?sort=lastupdate&page=1&lang=zh_CN
```

### `GET /category`

按分类标签获取小说列表。

| 参数 | 类型 | 必填 | 默认值 |
| --- | --- | --- | --- |
| `tag` | string | 是 | - |
| `sort` | `NovelSortMethod` | 是 | - |
| `page` | integer | 否 | `1` |
| `lang` | string | 否 | `zh_CN` |

```http
GET /category?tag=%E5%A5%87%E5%B9%BB&sort=lastupdate&page=1
```

### `GET /finished`

获取已完结小说列表。

参数：可选 `page` 和可选 `lang`。

```http
GET /finished?page=1&lang=zh_CN
```

## 8. 用户与书架接口

### `GET /bookshelf`

获取当前登录用户的书架。

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `bid` | integer | 否 | `0` | 书架分类 ID，最小为 0 |
| `lang` | string | 否 | `zh_CN` | 输出语言 |

成功响应：`BookshelfItem[]`。

### `GET /user/bookshelf/{uid}`

获取指定用户公开书架中的封面条目。虽然目标是其他用户，该接口当前仍要求服务登录。

| 参数 | 位置 | 类型 | 必填 | 默认值 |
| --- | --- | --- | --- | --- |
| `uid` | path | integer | 是 | - |
| `lang` | query | string | 否 | `zh_CN` |

成功响应：`NovelCover[]`。

### `GET /user/info`

获取当前共享登录用户的信息。

参数：可选 `lang`。

成功响应：`UserInfo`。注意该响应包含邮箱，服务不应直接暴露给不可信网络。

## 9. 评论与推荐接口

### `GET /novel/{aid}/comments`

获取小说书评。

| 参数 | 位置 | 类型 | 必填 | 默认值 |
| --- | --- | --- | --- | --- |
| `aid` | path | integer | 是 | - |
| `page` | query | integer | 否 | `1` |
| `lang` | query | string | 否 | `zh_CN` |

成功响应：`CommentItem[]`。

### `GET /comments/{rid}/replies`

获取指定书评的回复。

| 参数 | 位置 | 类型 | 必填 | 默认值 |
| --- | --- | --- | --- | --- |
| `rid` | path | integer | 是 | - |
| `page` | query | integer | 否 | `1` |
| `lang` | query | string | 否 | `zh_CN` |

成功响应：`ReplyItem[]`。

### `GET /recommend`

获取首页推荐区块。参数为可选 `lang`，成功响应为 `RecommendBlock[]`。

## 10. 图片透传接口

### `GET /picture`

透传 Wenku8 域内图片，用于加载章节插图或远程封面。

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `url` | string | 是 | 完整图片 URL，必须进行 URL 编码 |

仅允许主机名以 `wenku8.com` 或 `wenku8.net` 结尾，其他域名返回 HTTP 400。

```http
GET /picture?url=https%3A%2F%2Fimg.wenku8.com%2Fimage%2F1%2F1587%2F1587s.jpg
```

成功响应为 JPEG 二进制。

## 11. 数据模型

以下使用接近 TypeScript 的形式描述 JSON。`?` 表示字段可能为 `null`。

```typescript
interface NovelInfo {
  aid: number;
  title: string;
  author: string;
  status: string;
  last_updated: string | null;
  intro: string;
  tags: string[];
  press: string;
  word_count: number | null;
  popularity_level: string | null;
  trending_level: string | null;
  latest_section: string | null;
  copyright: boolean;
  animation: boolean;
}

interface Chapter {
  cid: number;
  title: string;
}

interface Volume {
  vid: number;
  title: string;
  chapters: Chapter[];
}

interface NovelIndex {
  aid: number;
  title: string;
  author: string;
  volumes: Volume[];
}

interface SearchItem {
  aid: number;
  title: string;
  author: string;
  press: string;
  last_updated: string | null;
  word_count: string | null;
  status: string;
  tags: string[];
  intro_preview: string;
  copyright: boolean;
  animation: boolean;
}

interface PageControl {
  now: number;
  previous: number;
  next: number;
  begin: number;
  end: number;
}

interface SearchResult {
  results: SearchItem[];
  page_control: PageControl;
}

interface BookshelfItem {
  aid: number;
  bid: number;
  title: string;
  author: string;
  latest_section: string;
  latest_section_cid: number;
  bookmark: string | null;
  bookmark_cid: number | null;
  last_updated: string;
  finished: boolean;
  updated_after_last_reading: boolean;
}

interface NovelCover {
  title: string;
  aid: number;
  image_url: string | null;
}

interface CommentItem {
  rid: number;
  content: string;
  view_count: string;
  reply_count: string;
  user_name: string;
  uid: number;
  time: string;
}

interface ReplyItem {
  content: string;
  user_name: string;
  uid: number;
  time: string;
}

interface RecommendBlock {
  title: string;
  list: NovelCover[];
}

interface UserInfo {
  avatar: string;
  uid: number;
  username: string;
  user_level: string;
  email: string;
  register_date: string;
  contribution: string;
  experience: string;
  point: string;
  max_bookshelf_num: string;
  max_recommend_num: string;
}
```

兼容性提示：模型契约中 `aid` 是整数，但当前部分搜索解析路径可能输出数字字符串（例如 `"1587"`）。Rust 客户端建议在反序列化层兼容数字和数字字符串，或等待服务端统一类型。

## 12. 错误响应

业务异常统一采用以下 JSON：

```json
{
  "detail": "错误描述",
  "type": "错误类型"
}
```

| HTTP 状态 | `type` | 含义 |
| --- | --- | --- |
| 400 | `invalid_url` | `/picture` URL 不属于 Wenku8 域名 |
| 401 | `not_logged_in` | 服务启动登录失败或登录态已失效 |
| 422 | FastAPI 校验错误 | 缺少参数、枚举错误或数值越界 |
| 429 | `rate_limited` | Wenku8/Cloudflare 对出口 IP 限流 |
| 502 | `cloudflare_challenge` | 上游返回 Cloudflare 质询 |
| 502 | `page_parse_error` | 上游页面结构与解析器预期不一致 |
| 502 | 具体异常类名 | CDN、网络或其他未分类上游错误 |

未登录示例：

```json
{
  "detail": "",
  "type": "not_logged_in"
}
```

客户端建议：

1. 收到 401 后调用 `/health`，提示用户重启或修复服务登录。
2. 对 429 和 502 使用有限次数的指数退避，不要立即高频重试。
3. 对 422 直接修正请求参数，不要重试。
4. 图片和正文请求应设置合理的连接、读取超时。

## 13. 推荐客户端调用流程

```text
GET /health
  -> logged_in=false：仅使用封面/整本下载，提示登录态不可用
  -> logged_in=true：继续

GET /search/by_name?keyword=...
  -> 从 results[].aid 选择小说

GET /novel/info/{aid}
GET /novel/cover/{aid}
GET /novel/index/{aid}
  -> 从 volumes[].chapters[].cid 选择章节

GET /novel/content/{aid}/{cid}
  -> 渲染文本
  -> 遇到 <!--image-->URL<!--image--> 时调用 /picture?url=...
```

## 14. 并发与部署注意事项

- 服务内部共享单个浏览器和单个登录态。
- 所有浏览器页面导航会被串行化；并发请求不会并行抓取多个 Wenku8 页面。
- 封面、图片和整本 TXT 使用独立 HTTP 下载路径，可以并发。
- 搜索调用具有至少 5 秒间隔的冷却机制。
- 服务当前没有面向客户端的身份认证与权限隔离，且 `/user/info` 会返回账号邮箱；不要直接暴露到公网。
- 当前未配置 CORS。普通浏览器前端跨域访问时需要同源代理或在服务端增加 CORS；Tauri 原生 HTTP 客户端通常不受浏览器 CORS 限制。

