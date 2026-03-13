# GitHub Actions 多平台编译指南

本文档说明如何在 GitHub 上编译 Zed 编辑器的多平台安装包。

## 概述

我为你创建了两个工作流文件：

1. **`.github/workflows/build-simple.yml`** - 简化版工作流
   - 更容易理解和使用
   - 支持 macOS (Intel + Apple Silicon)、Linux x86_64、Windows x86_64
   - 无需特殊 secrets 配置

2. **`.github/workflows/build-all-platforms.yml`** - 完整版工作流
   - 支持所有6个平台：macOS (x86_64/aarch64)、Linux (x86_64/aarch64)、Windows (x86_64/aarch64)
   - 更复杂的配置，但有更多功能

## 快速开始

### 方法一：通过 GitHub 网页手动触发

1. 打开你的 GitHub 仓库页面：`https://github.com/18701745572/codej`
2. 点击上方的 **Actions** 标签
3. 在左侧工作流列表中选择 **Build Simple**（或 **Build All Platforms**）
4. 点击右侧的 **Run workflow** 按钮
5. 选择发布通道（stable/preview/nightly/dev），然后点击 **Run workflow**
6. 等待构建完成（通常需要 30-60 分钟）
7. 完成后，在 Actions 页面下载构建产物

### 方法二：通过推送标签自动触发

```bash
# 创建并推送标签
git tag v0.1.0
git push origin v0.1.0
```

推送标签后，工作流会自动运行，并在完成后创建 GitHub Release。

## 各平台说明

### macOS

- **架构**: 同时支持 Intel (x86_64) 和 Apple Silicon (aarch64)
- **输出格式**: `.dmg` 文件
- **代码签名**: 默认未签名（需要 Apple Developer 证书才能签名）
- **运行器**: `macos-latest` (Apple Silicon), `macos-13` (Intel)

### Linux

- **架构**: x86_64 和 aarch64
- **输出格式**: `.tar.gz` 压缩包
- **依赖**: 构建时需要 `libasound2-dev`, `libfontconfig1-dev`, `libwayland-dev` 等
- **注意**: aarch64 跨平台编译可能需要额外配置

### Windows

- **架构**: x86_64 和 aarch64
- **输出格式**: `.exe` 安装程序或 `.zip` 压缩包
- **要求**: 需要 Visual Studio 构建工具
- **代码签名**: 默认未签名

## 重要注意事项

### 1. 代码签名

默认的工作流不包含代码签名，这意味着：

- **macOS**: 用户打开应用时会看到 "无法验证开发者" 的警告，需要在系统设置中允许
- **Windows**: SmartScreen 可能会拦截未签名的安装程序
- **Linux**: 无影响

如需代码签名，需要：
- macOS: Apple Developer 证书（$99/年）
- Windows: 代码签名证书（可从证书颁发机构购买）

### 2. 构建时间

完整的多平台构建通常需要：
- macOS: 20-40 分钟
- Linux: 15-30 分钟
- Windows: 20-40 分钟

### 3. 构建失败常见原因

| 问题 | 解决方案 |
|------|----------|
| 缺少依赖 | 检查是否正确安装了系统依赖 |
| Rust 编译错误 | 确保使用正确的 Rust 版本（查看 `rust-toolchain.toml`） |
| 内存不足 | GitHub Actions 免费版有资源限制 |
| 跨平台编译失败 | Linux aarch64 需要额外配置交叉编译工具链 |

### 4. 与原 Zed 仓库的区别

原 Zed 仓库使用：
- 私有运行器（`namespace-profile-*`, `self-32vcpu-windows-2022`）
- Sentry 集成（错误追踪）
- 代码签名（Azure 和 Apple）
- 自定义 blob 存储

你的 fork 使用：
- GitHub 提供的标准运行器（免费）
- 无 Sentry 集成
- 无代码签名
- 产物存储在 GitHub Artifacts

## 自定义配置

### 修改发布通道

编辑工作流文件中的这一行：

```yaml
run: echo "stable" > crates/zed/RELEASE_CHANNEL
```

可改为：`stable`, `preview`, `nightly`, `dev`

### 添加代码签名（macOS）

在工作流中添加以下 secrets：
- `MACOS_CERTIFICATE`: Base64 编码的 .p12 证书
- `MACOS_CERTIFICATE_PASSWORD`: 证书密码
- `APPLE_NOTARIZATION_KEY`: Apple API 密钥
- `APPLE_NOTARIZATION_KEY_ID`: API 密钥 ID
- `APPLE_NOTARIZATION_ISSUER_ID`: Issuer ID

然后在仓库设置中添加这些 secrets。

### 修改触发条件

编辑工作流文件的 `on:` 部分：

```yaml
on:
  push:
    branches: [main]  # main 分支推送时触发
  pull_request:       # PR 时触发
  schedule:
    - cron: '0 0 * * 0'  # 每周日触发
  workflow_dispatch:  # 手动触发
```

## 故障排除

### 检查构建日志

1. 进入 Actions 页面
2. 点击失败的 workflow run
3. 查看具体 job 的日志

### 本地测试构建脚本

在提交前，你可以在本地测试构建：

```bash
# macOS
./script/bundle-mac aarch64-apple-darwin

# Linux
./script/bundle-linux

# Windows (在 PowerShell 中)
./script/bundle-windows.ps1 -Architecture x86_64
```

## 参考链接

- [Zed 官方文档](https://zed.dev/docs)
- [GitHub Actions 文档](https://docs.github.com/actions)
- [cargo-bundle 文档](https://github.com/burtonageo/cargo-bundle)
- [Inno Setup 文档](https://jrsoftware.org/isinfo.php)

## 需要帮助？

如果在构建过程中遇到问题：

1. 检查 GitHub Actions 日志获取详细错误信息
2. 查看 [Zed 官方仓库](https://github.com/zed-industries/zed) 的 issues
3. 确保你的 fork 与原仓库同步（获取最新的修复）
