# OpenTimeSync

[English](#english) | [中文](#chinese)

---

<a name="english"></a>

# OpenTimeSync

Cross-platform high-precision time synchronization desktop app via NTP-weighted clock.

Built with **Tauri + Rust** — runs on **Windows, macOS, Linux**. Native WebView2 (no bundled Chromium).

> **~5MB** binary, compared to ~200MB for Electron — instant startup, minimal resource usage.

## Features

- **NTP Server Selection** — Choose from 5 NTP servers (Tencent/Aliyun/Apple/Google/Pool). Default: ntp.tencent.com
- **High-Precision Sync** — Weighted averaging with outlier filtering, precision grading S+ to D
- **NTP Offset Display** — Shows real deviation between local clock and NTP time
- **Network Latency** — Per-server RTT displayed in real-time
- **14 Timezones** — Quick switch with cyberpunk dropdown panel
- **Cyberpunk UI** — Dark theme with neon glow, glass morphism, flowing light animation
- **Responsive Layout** — Scales smoothly from 600px to 4K
- **Live UTC Display** — Continuous millisecond-accurate time rendering

## Download

Get the latest build from [GitHub Releases](https://github.com/Qiguiqiang/timeSyncWord/releases):

| Platform | File | Type |
|----------|------|------|
| Windows | `OpenTimeSync_*_x64_en-US.msi` | MSI Installer |
| macOS | `OpenTimeSync_*.dmg` | DMG Installer |
| Linux | `opentimesync_*_amd64.deb` | Deb package |
| Linux | `OpenTimeSync_*.AppImage` | Portable (no install) |

## Quick Start (Development)

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (22+)

Linux additionally requires:
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf
```

### Run

```bash
npm install
npm run dev
```

### Build

```bash
npm run build
# Output: src-tauri/target/release/bundle/
```

## Project Structure

```
OpenTimeSync/
├── src-tauri/            # Rust backend (Tauri + NTP)
│   ├── src/
│   │   ├── main.rs       # Tauri entry point
│   │   ├── lib.rs        # Commands, sync loop, event system
│   │   └── ntp.rs        # NTP protocol (UDP, offset/RTT calculation)
│   ├── Cargo.toml        # Rust dependencies
│   ├── tauri.conf.json   # Window config, bundle settings
│   └── capabilities/     # Tauri v2 permission grants
├── public/               # Frontend UI
│   ├── index.html        # Main page
│   ├── css/style.css     # Cyberpunk theme
│   └── js/app.js         # Time sync + NTP/TZ selectors
├── build/                # App icons (PNG/SVG)
├── .github/workflows/    # CI/CD: auto-build Win/Mac/Linux
└── package.json          # Tauri CLI dependency
```

## Precision Grades

| Grade | Offset Std Dev | Description |
|-------|---------------|-------------|
| S+ | < 2ms | Extremely stable |
| S | < 5ms | Very stable |
| S- | < 10ms | Stable |
| A | < 30ms | Good |
| B | < 50ms | Fair |
| C | < 100ms | Poor |
| D | >= 100ms | Unstable |

## CI/CD

Push a tag to trigger automatic cross-platform builds:

```bash
git tag v1.0.0
git push origin v1.0.0
```

GitHub Actions compiles Windows (MSI), macOS (DMG), and Linux (deb + AppImage).

## Architecture

```
NTP Server (selected) → Rust (tokio UDP) → Tauri Event → System WebView
                              ↓
                     NTP offset + RTT displayed in UI
```

All processing is local — no external server dependency. The Rust backend handles
NTP queries via raw UDP sockets, calculates offset and RTT, and pushes time updates
to the frontend via Tauri's event system.

## Configuration

NTP servers and sync parameters are defined in `src-tauri/src/lib.rs`:

| Constant | Default | Description |
|----------|---------|-------------|
| `NTP_SERVERS` | 5 servers | Available NTP server list |
| First server | ntp.tencent.com | Default active server |
| Sync interval | 2 seconds | NTP query interval |
| Timeout | 2 seconds | Per-query UDP timeout |

---

<a name="chinese"></a>

# OpenTimeSync

基于 NTP 加权时钟的高精度时间同步跨平台桌面应用。

基于 **Tauri + Rust** 构建，支持 **Windows、macOS、Linux**，使用系统原生 WebView2（不内嵌 Chromium）。

> **~5MB** 二进制文件，对比 Electron 约 200MB——瞬间启动、极低资源占用。

## 功能特性

- **NTP 服务器选择** — 5 个 NTP 服务器可选（腾讯云/阿里云/Apple/Google/Pool），默认 ntp.tencent.com
- **高精度同步** — 加权平均 + 异常值过滤，精度等级 S+ 到 D
- **NTP 偏差显示** — 实时展示本机时钟与 NTP 标准时间的偏差
- **网络延迟** — 每个服务器的实时 RTT 延迟显示
- **14 个时区** — 赛博朋克下拉面板快速切换
- **赛博朋克 UI** — 暗色主题、霓虹发光、玻璃拟态、流水光效
- **响应式布局** — 从 600px 到 4K 流畅缩放
- **实时 UTC** — 毫秒级精度持续时间渲染

## 下载

最新编译版本在 [GitHub Releases](https://github.com/Qiguiqiang/timeSyncWord/releases) 页面：

| 平台 | 文件 | 类型 |
|----------|------|------|
| Windows | `OpenTimeSync_*_x64_en-US.msi` | MSI 安装包 |
| macOS | `OpenTimeSync_*.dmg` | DMG 安装包 |
| Linux | `opentimesync_*_amd64.deb` | Deb 包 |
| Linux | `OpenTimeSync_*.AppImage` | 便携版（无需安装） |

## 快速开始（开发模式）

### 环境要求

- [Rust](https://rustup.rs/)（stable）
- [Node.js](https://nodejs.org/)（22+）

Linux 额外依赖：
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf
```

### 运行

```bash
npm install
npm run dev
```

### 打包

```bash
npm run build
# 输出: src-tauri/target/release/bundle/
```

## 项目结构

```
OpenTimeSync/
├── src-tauri/            # Rust 后端（Tauri + NTP）
│   ├── src/
│   │   ├── main.rs       # Tauri 入口
│   │   ├── lib.rs        # 命令、同步循环、事件系统
│   │   └── ntp.rs        # NTP 协议（UDP、偏差/RTT 计算）
│   ├── Cargo.toml        # Rust 依赖
│   ├── tauri.conf.json   # 窗口配置、打包设置
│   └── capabilities/     # Tauri v2 权限授权
├── public/               # 前端 UI
│   ├── index.html        # 主页面
│   ├── css/style.css     # 赛博朋克主题
│   └── js/app.js         # 时间同步 + NTP/时区选择器
├── build/                # 应用图标（PNG/SVG）
├── .github/workflows/    # CI/CD: 自动编译 Win/Mac/Linux
└── package.json          # Tauri CLI 依赖
```

## 精度等级

| 等级 | 偏移标准差 | 说明 |
|-------|---------------|-------------|
| S+ | < 2ms | 极其稳定 |
| S | < 5ms | 非常稳定 |
| S- | < 10ms | 稳定 |
| A | < 30ms | 良好 |
| B | < 50ms | 一般 |
| C | < 100ms | 较差 |
| D | >= 100ms | 不稳定 |

## CI/CD 自动编译

推送标签自动触发全平台编译：

```bash
git tag v1.0.0
git push origin v1.0.0
```

GitHub Actions 自动编译 Windows（MSI）、macOS（DMG）和 Linux（deb + AppImage）。

## 架构

```
选择的 NTP 服务器 → Rust（tokio UDP）→ Tauri 事件 → 系统 WebView
                              ↓
                     显示 NTP 偏差 + RTT 延迟
```

所有处理在本地完成，无需外部服务器。Rust 后端通过原始 UDP socket 执行 NTP 查询，
计算偏差和 RTT，通过 Tauri 事件系统向前端推送时间更新。

## 配置

NTP 服务器和同步参数在 `src-tauri/src/lib.rs` 中定义：

| 常量 | 默认值 | 说明 |
|----------|---------|-------------|
| `NTP_SERVERS` | 5 台服务器 | 可用 NTP 服务器列表 |
| 第一台 | ntp.tencent.com | 默认 NTP 服务器 |
| 同步间隔 | 2 秒 | NTP 查询间隔 |
| 超时 | 2 秒 | 单次 UDP 超时 |

## 许可证

MIT
