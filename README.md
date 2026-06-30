# 堡垒机一键 RDP 连接工具

通过亚信堡垒机（iFort SSO）一键连接远程桌面，无需打开网页、手动复制 token。

## 功能

- 双击启动，图形界面配置
- SM2 加密登录，安全传输密码
- 自动获取 SSO 令牌，一键拉起远程桌面
- 配置加密存储（Windows DPAPI），仅本机可解密

## 前置依赖

本程序依赖 Node.js 的 `sm-crypto` 模块做 SM2 加密。需要确保 `node_modules/sm-crypto` 存在于以下路径之一：

- `bastion_rdp.exe` 同目录下的 `node_modules/sm-crypto/`
- `C:/Users/admin/AppData/Local/Temp/opencode/node_modules/sm-crypto/`
- `C:/Users/admin/Desktop/node_modules/sm-crypto/`

如未安装，在 exe 同目录执行：

```bash
npm install sm-crypto
```

## 使用方式

双击 `bastion_rdp.exe`：

1. **首次运行** → 弹出配置窗口，填写：
   - 堡垒机用户名
   - 堡垒机密码
   - 服务器密码
   - 服务器账号（默认同堡垒机用户名）
   - 服务器 IP（默认 `10.237.35.254`）
   
   点"保存并连接"

2. **后续运行** → 弹出两个按钮：
   - "使用上次账号密码连接"
   - "修改账号密码"

   连接成功后自动弹出 Windows 远程桌面。

## 工作原理

```
用户输入 → SM2 加密 → 调用堡垒机 API 登录 → 
获取 SSO 令牌 → 调用 AsiainfoSSO_x64.exe → 
建立 SSH 隧道 → 拉起 mstsc.exe 远程桌面
```

## 开发

### 项目结构

```
src/
  main.rs      - GUI 入口（egui）
  api.rs       - 堡垒机 API 调用（reqwest）
  crypto.rs    - SM2 加密（调用 Node.js sm-crypto）
  config.rs    - 配置读写
```

### 编译

```bash
cargo build --release
```

输出：`target/release/bastion_rdp.exe`

### 依赖

```toml
egui = "0.31"       # 图形界面
eframe = "0.31"     # egui 框架
reqwest = "0.12"    # HTTP 客户端
tokio = "1"         # 异步运行时
serde/serde_json    # JSON 序列化
```

## 许可证

MIT
