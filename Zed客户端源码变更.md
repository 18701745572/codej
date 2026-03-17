# Zed 客户端源码变更记录

本文档记录为对接 CodeJ（codej.cn）而对 Zed 客户端源码所做的修改。

## 一、变更概览

| 文件 | 新增 | 删除 | 净增 |
|------|------|------|------|
| `assets/settings/default.json` | 1 | 1 | 0 |
| `crates/client/src/client.rs` | ~120 | ~3 | ~117 |
| `crates/client/src/user.rs` | ~85 | 0 | ~85 |
| `crates/cloud_api_client/src/cloud_api_client.rs` | ~160 | 1 | ~159 |
| `crates/cloud_api_types/src/cloud_api_types.rs` | 17 | 0 | 17 |
| `crates/zed/src/codej_login_modal.rs` | ~180 | 0 | ~180 |
| `crates/zed/src/main.rs` | ~25 | ~1 | ~24 |
| `crates/zed/src/zed.rs` | 1 | 0 | 1 |
| **合计** | **~589** | **~6** | **~583** |

共修改 8 个文件，净增约 583 行代码。

---

## 二、详细变更说明

### 2.1 默认服务器地址

**文件**：`assets/settings/default.json`

**变更**：将默认 `server_url` 从 `https://zed.dev` 改为 `https://codej.cn`

```json
"server_url": "https://codej.cn",
```

---

### 2.2 云 API 类型定义

**文件**：`crates/cloud_api_types/src/cloud_api_types.rs`

**新增类型**：

- **`UserPreferences`**：用户模型偏好
  - `default_model`：默认模型
  - `inline_assistant_model`：内联助手模型

- **`ModelSelection`**：模型选择
  - `provider`：提供商标识
  - `model`：模型 ID

- **`UserApiKeys`**：API Key 映射（`HashMap<String, String>`），key 为 provider，value 为 API Key

---

### 2.3 云 API 客户端

**文件**：`crates/cloud_api_client/src/cloud_api_client.rs`

**新增方法**：

1. **`get_user_preferences()`**
   - 请求 `GET /client/user_preferences`
   - 返回 `Result<Option<UserPreferences>>`
   - 404、501、401 时返回 `Ok(None)`，静默跳过

2. **`get_user_api_keys()`**
   - 请求 `GET /client/api_keys`
   - 返回 `Result<Option<UserApiKeys>>`
   - 404、501、401 时返回 `Ok(None)`，静默跳过

3. **`codej_login(email, password)`**
   - 请求 `POST /api/auth/login`，Body: `{ email, password }`
   - 返回 `Result<CodeJAuthResponse>`，包含 `user_id`、`access_token`
   - 用于 CodeJ 邮箱密码直连登录

4. **`codej_register(email, password)`**
   - 请求 `POST /api/auth/register`，Body: `{ email, password }`
   - 返回 `Result<CodeJAuthResponse>`
   - 用于 CodeJ 邮箱密码注册

---

### 2.4 登录回调支持明文 token（CodeJ）

**文件**：`crates/client/src/client.rs`

**变更**：支持 CodeJ 使用明文 token 的登录回调，避免 Node.js 与 Rust RSA 实现差异导致解密失败。

1. **`CallbackParams` 新增 `encrypted` 字段**
   - 可选参数，用于标识 `access_token` 是否已加密
   - `encrypted=false` 时表示明文 token，无需解密

2. **解密逻辑调整**
   - 当 `encrypted=Some("false")` 时，直接使用 `access_token` 作为凭据
   - 否则按原逻辑使用 RSA 私钥解密（兼容 zed.dev）

---

### 2.5 CUID 用户 ID 支持（CodeJ）

**文件**：`crates/client/src/client.rs`、`crates/cloud_api_client/src/cloud_api_client.rs`

**变更**：CodeJ 使用 CUID 字符串作为用户 ID，而 Zed 原设计使用数字 ID。为使 `Authorization` 请求头与 CodeJ 的 `verifyAccessToken` 匹配，需在 API 请求中传递原始 CUID。

1. **`Credentials` 新增 `user_id_for_header` 字段**
   - `Option<String>`：CodeJ 时为 `Some(cuid)`，Zed 时为 `None`
   - 用于 `Authorization` 请求头及凭据存储
   - 新增 `user_id_for_api()` 方法，返回用于 API 的 user_id 字符串

2. **回调解析逻辑**
   - 若 `user_id` 可解析为 `u64`（Zed）：`user_id_for_header = None`
   - 若为 CUID（CodeJ）：`user_id_for_header = Some(user_id)`，内部 `user_id` 由 `cuid_to_user_id()` 计算

3. **`cloud_api_client` 变更**
   - `Credentials` 内部 `user_id` 改为 `String`，支持任意字符串
   - `set_credentials(user_id: impl Into<String>, access_token)`：接受字符串形式 user_id
   - `validate_credentials(user_id: impl Into<String>, access_token)`：同上

4. **凭据读写**
   - `write_credentials` 改为接受 `impl AsRef<str>`，存储 `user_id_for_api()` 的返回值
   - `read_credentials` 解析存储的字符串：数字则 `user_id_for_header = None`，否则视为 CUID

---

### 2.6 CodeJ API 直连登录与注册

**文件**：`crates/client/src/client.rs`、`crates/zed/src/codej_login_modal.rs`、`crates/zed/src/main.rs`

**变更**：当 `server_url` 指向 CodeJ 时，用户可本地联网通过邮箱密码直接登录，无需打开浏览器。

1. **`init_with_codej_handler(client, cx, codej_sign_in_handler)`**
   - 扩展 `client::init`，支持传入 CodeJ 登录回调
   - 当 Sign In 被触发且 `is_codej_server(server_url)` 时，优先调用回调而非浏览器流程

2. **`sign_in_with_api_credentials(email, password, cx)`**
   - 调用 `cloud_client.codej_login` 获取凭据
   - 写入凭据、更新状态、连接 Cloud，流程与 `sign_in` 一致

3. **`sign_in_with_api_register(email, password, cx)`**
   - 调用 `cloud_client.codej_register` 注册并获取凭据
   - 后续流程同 `sign_in_with_api_credentials`

4. **`is_codej_server(server_url)`**
   - 从 `user.rs` 导出为 `pub`，供 init 判断是否走 CodeJ 流程

5. **CodeJLoginModal**
   - 新增模态框：邮箱、密码输入框，登录、注册、使用浏览器登录按钮
   - 点击登录/注册后调用对应 API，成功后关闭模态框

6. **无窗口时回退**
   - 若无活跃窗口（如应用刚启动），CodeJ 回调会回退到浏览器登录流程

---

### 2.7 客户端用户模块（CodeJ 同步逻辑）

**文件**：`crates/client/src/user.rs`

**新增**：

1. **`is_codej_server(server_url)`**
   - 判断 `server_url` 是否为 CodeJ 服务器（非 zed.dev、staging、localhost）

2. **`PROVIDER_API_URLS`**
   - Provider 与 API URL 映射：openai、anthropic、deepseek、kimi

3. **`sync_from_codej(cloud_client, cx)`**
   - 当 `server_url` 指向 CodeJ 时，在 `get_authenticated_user` 成功后执行：
     - 拉取用户偏好，写入 `agent.default_model`、`agent.inline_assistant_model`
     - 拉取 API Key，通过 CredentialsProvider 写入系统凭据

**调用时机**：在 `get_authenticated_user` 成功返回后、更新当前用户状态之前

---

## 三、涉及文件列表

| 文件路径 | 变更类型 |
|----------|----------|
| `assets/settings/default.json` | 修改默认配置 |
| `crates/client/src/client.rs` | 登录回调支持 `encrypted=false`；Credentials 新增 `user_id_for_header`；CodeJ API 登录/注册；`init_with_codej_handler` |
| `crates/cloud_api_types/src/cloud_api_types.rs` | 新增类型定义 |
| `crates/cloud_api_client/src/cloud_api_client.rs` | 新增 API 方法；`codej_login`/`codej_register`；`set_credentials`/`validate_credentials` 支持字符串 user_id |
| `crates/client/src/user.rs` | 新增 CodeJ 同步逻辑；`is_codej_server` 导出为 pub |
| `crates/zed/src/codej_login_modal.rs` | 新增 CodeJ 登录模态框 |
| `crates/zed/src/main.rs` | 使用 `init_with_codej_handler` 注册 CodeJ 登录回调 |
| `crates/zed/src/zed.rs` | 新增 `codej_login_modal` 模块 |

---

## 四、与 CodeJ 的配合说明

### 4.1 CodeJ 需实现的 API

- **`GET /client/users/me`**（必需）：返回 Zed 兼容的 `GetAuthenticatedUserResponse` 格式，包含 `user`、`feature_flags`、`organizations`、`plan` 等，否则 Zed 无法解析响应、标题栏仍显示 Sign In。CodeJ 实现见 `codej.cn/app/client/users/me/route.ts`
- `GET /client/user_preferences`：返回用户模型偏好
- `PUT /client/user_preferences`：更新用户模型偏好
- `GET /client/api_keys`：返回用户 API Key（明文，供 Zed 拉取后写入本地）
- `PUT /client/api_keys`：更新用户 API Key
- **`POST /api/auth/login`**（API 登录）：Body `{ email, password }`，返回 `{ user_id, access_token }`。需支持跨域（CORS）以便 Web 版调用
- **`POST /api/auth/register`**（API 注册）：Body `{ email, password }`，返回 `{ user_id, access_token }`。同上需支持 CORS

### 4.2 登录回调格式（CodeJ）

CodeJ 使用明文 token 回调，需在 `redirectUrl` 中附带 `encrypted=false`：

```
http://127.0.0.1:{port}?user_id={user_id}&access_token={plain_token}&encrypted=false
```

- `user_id`：CodeJ 用户 ID（CUID 字符串）
- `access_token`：明文 access_token（无需 RSA 加密）
- `encrypted=false`：标识 token 为明文，客户端将直接使用

### 4.3 回调 user_id 与 Authorization 兼容性

- **回调解析**：`user_id` 可解析为 `u64` 时（Zed）使用数字；否则视为 CUID（CodeJ），通过 `cuid_to_user_id()` 计算内部 `user_id`，并存储 `user_id_for_header: Some(cuid)`
- **API 请求**：`Authorization` 请求头格式为 `{user_id} {access_token}`。CodeJ 场景下必须传递原始 CUID，否则 `verifyAccessToken` 无法匹配数据库中的 token

### 4.4 API 直连登录流程（CodeJ）

当 `server_url` 为 codej.cn 时，用户点击 Sign In 会弹出 CodeJ 登录模态框：

1. **邮箱 + 密码登录**：输入邮箱和密码，点击「登录」，调用 `POST /api/auth/login`
2. **注册**：输入邮箱和密码，点击「注册」，调用 `POST /api/auth/register`
3. **使用浏览器登录**：点击「使用浏览器登录」，回退到原有 OAuth 回调流程

用户本地联网即可完成登录，无需依赖浏览器重定向。

---

## 五、参考文档

- CodeJ 产品需求：`codjweb/codej-product-requirements.md`
- PRD 第十一章：用户模型偏好同步
- PRD 第十二章：用户 API Key 同步
