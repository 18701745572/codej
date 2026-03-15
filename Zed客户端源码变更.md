# Zed 客户端源码变更记录

本文档记录为对接 CodeJ（codej.cn）而对 Zed 客户端源码所做的修改。

## 一、变更概览

| 文件 | 新增 | 删除 | 净增 |
|------|------|------|------|
| `assets/settings/default.json` | 1 | 1 | 0 |
| `crates/client/src/user.rs` | ~85 | 0 | ~85 |
| `crates/cloud_api_client/src/cloud_api_client.rs` | ~96 | 1 | ~95 |
| `crates/cloud_api_types/src/cloud_api_types.rs` | 17 | 0 | 17 |
| **合计** | **~199** | **~2** | **~197** |

共修改 4 个文件，净增约 197 行代码。

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

---

### 2.4 客户端用户模块（CodeJ 同步逻辑）

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
| `crates/cloud_api_types/src/cloud_api_types.rs` | 新增类型定义 |
| `crates/cloud_api_client/src/cloud_api_client.rs` | 新增 API 方法 |
| `crates/client/src/user.rs` | 新增 CodeJ 同步逻辑 |

---

## 四、与 CodeJ 的配合说明

### 4.1 CodeJ 需实现的 API

- `GET /client/user_preferences`：返回用户模型偏好
- `PUT /client/user_preferences`：更新用户模型偏好
- `GET /client/api_keys`：返回用户 API Key（明文，供 Zed 拉取后写入本地）
- `PUT /client/api_keys`：更新用户 API Key

### 4.2 回调 user_id 兼容性

**重要**：Zed 在登录回调中通过 `user_id.parse()?` 将 `user_id` 解析为 `u64`。CodeJ 当前使用 CUID 字符串，会导致解析失败。

**建议**：在 CodeJ 的 User 表中增加 `zedUserId`（数字类型），并在登录回调中使用该字段作为 `user_id` 返回。

---

## 五、参考文档

- CodeJ 产品需求：`codjweb/codej-product-requirements.md`
- PRD 第十一章：用户模型偏好同步
- PRD 第十二章：用户 API Key 同步
