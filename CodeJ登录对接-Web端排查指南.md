# CodeJ 登录对接 - Web 端排查指南

当 Zed 客户端报错 `failed to decrypt access token` 时，按以下步骤在 Web 端排查，确保加密输出符合客户端要求。

---

## 一、客户端要求速查表

| 项目 | 客户端实现 | Web 端需满足 |
|------|------------|--------------|
| 公钥格式 | `to_pkcs1_der()` → Base64URL 编码 | 必须能解析 **PKCS#1 DER** 格式 |
| 公钥 Base64 | `BASE64_URL_SAFE`（`-`/`_` 替代 `+`/`/`） | 解码时支持 URL-safe，含 padding 或无 padding |
| 加密方式 | 先试 OAEP-SHA256，再试 PKCS#1 v1.5 | 任选其一，当前建议 PKCS#1 v1.5 |
| 密文 Base64 | `URL_SAFE` 或 `URL_SAFE_NO_PAD` 解码 | 使用 `base64url` 编码输出 |
| access_token 明文 | 48 字节随机 → Base64URL，约 64 字符 | 任意 UTF-8 字符串，长度 ≤ 190 字节（OAEP） |

---

## 二、排查步骤

### 步骤 1：验证公钥解析

在 `login` 接口中，收到 `native_app_public_key` 后立即校验：

```typescript
// 在 encryptAccessToken 调用前添加
function verifyPublicKeyParsing(publicKeyBase64Url: string): void {
  const keyBuffer = parsePublicKey(publicKeyBase64Url);
  console.log("[DEBUG] 公钥 Base64 长度:", publicKeyBase64Url.length);
  console.log("[DEBUG] 公钥 DER 字节长度:", keyBuffer.length);
  
  try {
    const pkcs1Key = createPublicKey({
      key: keyBuffer,
      format: "der",
      type: "pkcs1",
    });
    console.log("[DEBUG] 公钥解析成功: PKCS#1");
  } catch (e1) {
    try {
      const spkiKey = createPublicKey({
        key: keyBuffer,
        format: "der",
        type: "spki",
      });
      console.log("[DEBUG] 公钥解析成功: SPKI");
    } catch (e2) {
      console.error("[DEBUG] 公钥解析失败:", e1, e2);
      throw new Error("无法解析客户端公钥");
    }
  }
}
```

**检查点**：确认至少一种解析方式成功，且无报错。

---

### 步骤 2：验证 Base64 编解码一致性

客户端使用 `URL_SAFE` / `URL_SAFE_NO_PAD`，对应 Base64URL（`-`、`_`，可带或不带 padding）：

```typescript
// 加密后验证
function verifyBase64Encoding(encrypted: Buffer): string {
  const base64url = encrypted.toString("base64url");
  // 客户端不接受标准 base64 的 + 和 /
  if (base64url.includes("+") || base64url.includes("/")) {
    throw new Error("密文必须使用 base64url，不能含 + 或 /");
  }
  return base64url;
}
```

**检查点**：密文只含 `A-Za-z0-9-_`，无 `+`、`/`、`=`（或仅末尾 padding）。

---

### 步骤 3：端到端自测（Node 内加解密）

在 Web 项目中加一个自测脚本，模拟「客户端公钥 → 加密 → 解密」全流程：

```typescript
// scripts/verify-rsa-encryption.ts
import { createPublicKey, publicEncrypt, privateDecrypt, constants } from "node:crypto";
import { generateKeyPairSync } from "node:crypto";

// 1. 模拟客户端：生成 PKCS#1 格式密钥对
const { publicKey: clientPublicKey, privateKey: clientPrivateKey } = generateKeyPairSync("rsa", {
  modulusLength: 2048,
  publicKeyEncoding: { type: "spki", format: "der" },
  privateKeyEncoding: { type: "pkcs8", format: "der" },
});

// 导出 PKCS#1 公钥（与 Zed 客户端 to_pkcs1_der 一致）
const spki = require("node:crypto").createPublicKey(clientPublicKey);
const pkcs1Der = spki.export({ type: "spki", format: "der" });
// 注意：Node 的 export 默认是 SPKI，需要转换。可用 node-forge 或 asn1.js 提取 PKCS#1 部分
// 简化：直接用 SPKI 测试，因为 CodeJ 已支持 SPKI 回退
const publicKeyBase64Url = Buffer.from(clientPublicKey).toString("base64url");

// 2. 模拟服务端：加密
const plainToken = "test_token_64_chars_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
const encrypted = publicEncrypt(
  { key: createPublicKey(clientPublicKey), padding: constants.RSA_PKCS1_PADDING },
  Buffer.from(plainToken, "utf8")
);
const encryptedBase64Url = encrypted.toString("base64url");

// 3. 模拟客户端：解密
const decrypted = privateDecrypt(
  { key: clientPrivateKey, padding: constants.RSA_PKCS1_PADDING },
  Buffer.from(encryptedBase64Url, "base64")
);
console.log("解密结果:", decrypted.toString() === plainToken ? "✓ 成功" : "✗ 失败");
```

**检查点**：Node 内加解密成功，说明算法和编码本身没问题。

---

### 步骤 4：检查 URL 传输是否破坏密文

在构建 `redirectUrl` 前后打日志：

```typescript
// login/route.ts 中
const encryptedToken = encryptAccessToken(accessToken, nativeAppPublicKey);

console.log("[DEBUG] 加密前 access_token 长度:", accessToken.length);
console.log("[DEBUG] 加密后密文长度:", encryptedToken.length);
console.log("[DEBUG] 密文前 20 字符:", encryptedToken.slice(0, 20));
console.log("[DEBUG] 密文含 + ?", encryptedToken.includes("+"));
console.log("[DEBUG] 密文含 / ?", encryptedToken.includes("/"));
console.log("[DEBUG] 密文含 = ?", encryptedToken.includes("="));

const callbackUrl = `http://127.0.0.1:${nativeAppPort}?user_id=${encodeURIComponent(user.id)}&access_token=${encodeURIComponent(encryptedToken)}`;

console.log("[DEBUG] redirectUrl 中 access_token 参数长度:", encodeURIComponent(encryptedToken).length);
```

**检查点**：
- 密文应使用 base64url，不含 `+`、`/`（`=` 仅作 padding 时可有）
- `encodeURIComponent` 后长度会略增（如 `-`→`%2D`），但解码后应还原

---

### 步骤 5：添加调试接口（可选）

临时增加一个接口，用固定明文测试加密，便于与客户端对照：

```typescript
// app/api/auth/debug-encrypt/route.ts
export async function POST(req: Request) {
  const { publicKeyBase64Url, plaintext } = await req.json();
  const encrypted = encryptAccessToken(plaintext, publicKeyBase64Url);
  return Response.json({
    encrypted,
    plaintextLength: plaintext.length,
    encryptedLength: encrypted.length,
  });
}
```

用 Postman 等工具调用，再用客户端私钥在本地解密验证。

---

## 三、常见问题对照

| 现象 | 可能原因 | 处理方式 |
|------|----------|----------|
| 公钥解析报错 | 收到的是 SPKI 或格式错误 | 先试 PKCS#1，再试 SPKI；确认 URL 未二次编码 |
| 解密失败 | 密文在 URL 中被篡改 | 使用 `encodeURIComponent`，iframe 用原始 `redirectUrl` |
| 解密失败 | 用了标准 Base64 | 改用 `buffer.toString("base64url")` |
| 解密失败 | OAEP 实现差异 | 优先使用 PKCS#1 v1.5 |
| 解密失败 | 公钥与私钥不匹配 | 确认 login 请求中的公钥与打开登录页时的公钥一致 |

---

## 四、客户端解密逻辑（供对照）

```rust
// crates/rpc/src/auth.rs
// 1. Base64 解码：支持 URL_SAFE（带 padding）或 URL_SAFE_NO_PAD
let encrypted_bytes = URL_SAFE.decode(encrypted_string)
    .or_else(|_| URL_SAFE_NO_PAD.decode(encrypted_string))?;

// 2. 解密顺序：先 OAEP-SHA256，失败则 PKCS#1 v1.5
let bytes = self.0.decrypt(oaep_sha256_padding(), &encrypted_bytes)
    .or_else(|_| self.0.decrypt(Pkcs1v15Encrypt, &encrypted_bytes))?;

// 3. 明文需为合法 UTF-8
let string = String::from_utf8(bytes)?;
```

---

## 五、关键：公钥来源一致性

**最容易出错的一点**：加密用的公钥必须与 Zed 打开登录页时 URL 中的 `native_app_public_key` 完全一致。

流程确认：
1. Zed 生成密钥对 → 用公钥拼出登录 URL
2. 用户打开该 URL → 前端从 URL 读取 `native_app_public_key`
3. 用户提交登录 → 前端把该公钥传给后端
4. 后端用该公钥加密 `access_token`

若前端从别处拿公钥（如缓存、默认值），或对公钥做了修改，解密必然失败。建议在 login 接口中打印收到的 `native_app_public_key` 前 50 字符，与打开登录页时浏览器地址栏中的值对比。

---

## 六、建议的最终检查清单

- [ ] 公钥能按 PKCS#1 或 SPKI 成功解析
- [ ] 加密使用 PKCS#1 v1.5（`RSA_PKCS1_PADDING`）
- [ ] 密文使用 `base64url`，不含 `+`、`/`
- [ ] `redirectUrl` 中 `user_id`、`access_token` 均经 `encodeURIComponent`
- [ ] 前端 iframe 使用 `data.redirectUrl` 原始字符串，不做二次编码
- [ ] **登录请求中的 `native_app_public_key` 与打开登录页时 URL 中的完全一致**

完成以上检查后，若仍报错，可抓取一次完整 login 请求/响应（含 `redirectUrl`）和客户端日志，便于进一步定位。
