# 堡垒机一键 RDP 连接工具

通过亚信堡垒机（iFort SSO）一键连接 Windows 远程桌面，无需手动打开网页、复制 token。

## 功能特性

- 双击启动，图形化界面配置
- SM2 国密加密登录，安全传输密码
- 自动获取 SSO 令牌，拉起本地远程桌面客户端
- 密码通过 Windows DPAPI 加密存储，仅本机用户可解密
- 纯 Rust 编译，单文件 `exe`，无运行时依赖

## 快速开始

1. 从 [Releases](https://github.com/Jakliuyuy/bastion_rdp_rust/releases) 下载 `bastion_rdp.exe`
2. 双击运行

### 首次使用

弹出配置窗口，填写：

| 字段 | 说明 | 示例 |
|------|------|------|
| 堡垒机用户名 | 登录堡垒机网页的账号 | `****` |
| 堡垒机密码 | 登录堡垒机网页的密码 | `******7` |
| 服务器密码 | 目标 Windows Server 的登录密码 | `******` |
| 服务器账号 | 目标服务器的用户名（默认同堡垒机用户名） | `*****` |
| 服务器 IP | 目标服务器的 IP 地址 | `*****` |

填写完成点击「保存并连接」。

### 后续使用

弹出主界面两个按钮：

- **使用上次账号密码连接** — 一键连接
- **修改配置** — 更改账号密码

连接成功后自动弹出 Windows 远程桌面窗口。

## 工作原理

```
┌─────────┐    SM2 加密     ┌──────────┐    获取令牌    ┌──────────┐   隧道转发   ┌──────────────┐
│  本机    │ ──────────────→ │  堡垒机   │ ─────────────→ │ Asiainfo │ ───────────→ │ 目标 Windows  │
│  exe    │ ←────────────── │  iFort    │ ←───────────── │  SSO     │ ←─────────── │  Server       │
└─────────┘    API 响应     └──────────┘    ifortsso://  └──────────┘   RDP 连接   └──────────────┘
```

1. 程序调用堡垒机 API，用 SM2 加密用户名和密码完成登录
2. 查询可用的 RDP 服务器列表，匹配配置的 IP
3. 调用堡垒机 SSO 接口，获取一次性连接令牌（`IfortSSO://`）
4. 通过系统协议处理器启动 `AsiainfoSSO_x64.exe`，传入令牌
5. 插件解密令牌，建立 SSH 隧道到堡垒机，再转发 RDP 到目标服务器
6. 自动拉起 `mstsc.exe`，用户即可操作远程桌面

## 安全说明

- 密码仅在内存中存在，配置文件 `%APPDATA%\bastion_rdp\config.json` 中存储的是经过 [Windows DPAPI](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/) 加密的密文
- 网络通信全程 HTTPS（自签名证书），登录凭证使用 SM2 公钥加密传输
- 每个会话的加密公钥由服务器临时生成，不同会话不同密钥

## 前置条件

1. **必须安装亚信 SSO 插件**（`AsiainfoSSO_x64.exe`），路径：
   ```
   C:\Program Files (x86)\IFORTSSO\AsiainfoSSO_x64.exe
   ```
   并且注册表中存在 `HKEY_CLASSES_ROOT\ifortsso` 协议处理器。

2. **服务器必须通过堡垒机可达**（本机无法直连目标服务器，必须走堡垒机隧道）。

3. 操作系统：Windows 10 / 11 x64

## 项目结构

```
src/
├── main.rs      # GUI 入口（egui）
├── api.rs       # 堡垒机 HTTP API 封装
├── crypto.rs    # SM2 加密（libsm）
└── config.rs    # 配置文件读写
```

## 开发编译

### 依赖

- Rust ≥ 1.70（MSVC 或 GNU toolchain）
- 首次编译需下载 `egui`、`reqwest`、`libsm` 等依赖

```bash
cargo build --release
```

输出：`target/release/bastion_rdp.exe`（约 5-8 MB）

### CI/CD

GitHub Actions 自动编译 Release，配置在 `.github/workflows/build.yml`：

```yaml
name: Build
on: [push, workflow_dispatch]
jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - uses: actions/upload-artifact@v4
        with:
          name: bastion_rdp
          path: target/release/bastion_rdp.exe
```

### 技术选型

| 组件 | 库 | 说明 |
|------|-----|------|
| GUI | `egui` + `eframe` | 纯 Rust 原生 GUI |
| HTTP | `reqwest` + `tokio` | 异步 HTTP 客户端 |
| SM2 加密 | `libsm` | 国密 SM2 椭圆曲线加密 |
| 配置加密 | `win32crypt` (旧版) / 计划改为纯 Rust | Windows DPAPI |

## 常见问题

### Q: 提示「参数解密错误」

检查 `config.json` 中的密码是否正确。如果之前存过旧版本配置，删除 `%APPDATA%\bastion_rdp\config.json` 后重新配置。

### Q: 提示「验证码为空」

堡垒机开启了滑块验证码。当前版本不支持验证码交互，需等验证码解除，或先通过浏览器登录一次后再使用本工具（会复用浏览器会话）。

### Q: 连接成功但没有弹出远程桌面

确认 `C:\Program Files (x86)\IFORTSSO\AsiainfoSSO_x64.exe` 存在且 `ifortsso` 注册表项正确。

### Q: 中文显示为方框

缺少中文字体。程序会自动加载 `C:/Windows/Fonts/msyh.ttc`，确保该文件存在。

## 许可证

MIT
