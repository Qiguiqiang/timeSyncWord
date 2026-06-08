# OpenTimeSync

**English** | [**中文**](#chinese)

Cross-platform high-precision NTP time synchronization desktop app. **~6MB** binary — built with **Tauri + Rust**, no bundled Chromium.

| Platform | Download |
|----------|----------|
| Windows | `OpenTimeSync_2.1.8_x64_en-US.msi` / `OpenTimeSync_2.1.8_x64-setup.exe` |
| macOS | `OpenTimeSync_2.1.8_x64.dmg` / `OpenTimeSync_2.1.8_aarch64.dmg` |
| Linux | `OpenTimeSync_2.1.8_amd64.deb` / `OpenTimeSync_2.1.8_amd64.AppImage` |

→ [Latest Release](https://github.com/Qiguiqiang/OpenTimeSync/releases)

## Features

- **5 NTP servers** — Tencent / Aliyun / Apple / Google / Pool, switchable at runtime
- **Weighted averaging** — outlier-removed, precision grades S+ to D
- **Tray + floating widget** — minimize or close to tray, restore from widget, draggable compact overlay
- **LAN sync topology** — local NTP / LAN master / LAN slave modes with pair code and master host settings
- **Settings panel** — configurable sync interval (2–3600s), auto-sync toggle, widget scale, sync mode
- **Startup calibration splash** — transparent boot mark and loading status while the first time baseline is established
- **macOS transparent overlay** — startup splash and floating widget use macOS private API for true transparent background outside the App Store path
- **Auto-update** — built-in updater checks GitHub Releases, shows release notes, download progress, and install handoff
- **14 timezones** — instant switch, persisted to localStorage
- **Cyberpunk UI** — neon glow, glass morphism, frameless window, millisecond-accurate time
- **CI/CD** — GitHub Actions auto-builds Win/Mac/Linux on tag push

## Quick Start

```bash
npm install
npm run dev        # dev mode with hot-reload
npm run build      # production build → src-tauri/target/release/bundle/
```

**Prerequisites:** Rust (stable), Node.js 22+. Linux additionally requires `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `patchelf`.

## Project Structure

```
OpenTimeSync/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs      # Entry point, windows_subsystem
│   │   ├── lib.rs       # Tauri commands, NTP sync loop, settings
│   │   └── ntp.rs       # NTPv3 UDP query, offset/RTT calculation
│   ├── Cargo.toml
│   └── tauri.conf.json
├── public/              # Frontend (no bundler)
│   ├── index.html
│   ├── css/style.css
│   └── js/app.js
├── .github/workflows/
└── package.json
```

## Architecture

```
NTP Server / LAN Master → Rust sync engine → offset/RTT computed → stored in AppState
                                                                   ↓
WebView / Widget ←─── invoke('get_ntp_status') every 1s ←─── frontend polls
```

The Rust backend queries the selected NTP server via raw UDP, or consumes the LAN master's calibrated time in slave mode, then computes offset and RTT using weighted averaging with outlier rejection. The frontend polls via Tauri IPC every second and renders corrected time with millisecond precision. All processing is local — no cloud dependency.

## Precision Grades

| Grade | Offset Std Dev |
|-------|---------------|
| S+ | < 2ms |
| S | < 5ms |
| S- | < 10ms |
| A | < 30ms |
| B | < 50ms |
| C | < 100ms |
| D | ≥ 100ms |

## CI/CD

Push a tag to trigger cross-platform builds:

```bash
git tag v2.1.8
git push origin v2.1.8
```

GitHub Actions produces: Windows MSI + NSIS, macOS DMG (x86_64 + aarch64), Linux deb + AppImage. A tagged release also updates GitHub Release assets and `updater.json`.

---

<a name="chinese"></a>

# OpenTimeSync

[**English**](#features) | [**中文**](#chinese)

跨平台高精度 NTP 时间同步桌面应用。**约 6MB** 二进制体积，基于 **Tauri + Rust** 构建，不内嵌 Chromium。

| 平台 | 下载 |
|----------|------|
| Windows | `OpenTimeSync_2.1.8_x64_en-US.msi` / `OpenTimeSync_2.1.8_x64-setup.exe` |
| macOS | `OpenTimeSync_2.1.8_x64.dmg` / `OpenTimeSync_2.1.8_aarch64.dmg` |
| Linux | `OpenTimeSync_2.1.8_amd64.deb` / `OpenTimeSync_2.1.8_amd64.AppImage` |

→ [最新 Release](https://github.com/Qiguiqiang/OpenTimeSync/releases)

## 功能特性

- **5 个 NTP 服务器** — 腾讯云 / 阿里云 / Apple / Google / Pool，运行时可切换
- **加权平均算法** — 自动剔除异常值，精度等级 S+ ~ D
- **托盘 + 悬浮挂件** — 最小化或关闭可收纳到托盘，支持悬浮挂件恢复主窗口与拖拽移动
- **局域网主从同步** — 支持本机 NTP / 局域网主机 / 局域网从机三种模式，带配对码与主机地址
- **设置面板** — 可配置同步间隔（2–3600s）、自动同步、挂件大小、同步模式
- **启动校准页** — 首轮建立时间基线时显示透明启动字样和加载状态
- **macOS 真透明悬浮层** — 启动页和悬浮挂件在 macOS 上启用私有 API 获取真正透明背景，不走 App Store 分发
- **自动更新** — 内置更新器检查 GitHub Releases，展示版本说明、下载进度与安装接力
- **14 个时区** — 即时切换，自动保存到 localStorage
- **赛博朋克 UI** — 霓虹发光、玻璃拟态、无边框窗口、毫秒级时间显示
- **CI/CD 自动编译** — GitHub Actions 推送标签即编译全平台安装包

## 快速开始

```bash
npm install
npm run dev        # 开发模式，支持热重载
npm run build      # 生产编译 → src-tauri/target/release/bundle/
```

**环境要求:** Rust（stable）、Node.js 22+。Linux 额外需要 `libwebkit2gtk-4.1-dev`、`libgtk-3-dev`、`libayatana-appindicator3-dev`、`librsvg2-dev`、`patchelf`。

## 项目结构

```
OpenTimeSync/
├── src-tauri/           # Rust 后端
│   ├── src/
│   │   ├── main.rs      # 入口点、windows_subsystem
│   │   ├── lib.rs       # Tauri 命令、NTP 同步循环、设置
│   │   └── ntp.rs       # NTPv3 UDP 查询、偏移/RTT 计算
│   ├── Cargo.toml
│   └── tauri.conf.json
├── public/              # 前端（无打包工具）
│   ├── index.html
│   ├── css/style.css
│   └── js/app.js
├── .github/workflows/
└── package.json
```

## 架构

```
NTP 服务器 / 局域网主机 → Rust 同步引擎 → 计算偏移/RTT → 存入 AppState
                                                             ↓
主界面 / 悬浮挂件 ←─── invoke('get_ntp_status') 每 1 秒 ←─── 前端轮询
```

Rust 后端通过原始 UDP socket 查询 NTP 服务器，或在从机模式下接收局域网主机的校准结果，使用加权平均 + 异常值剔除计算偏移和 RTT。前端通过 Tauri IPC 每 1 秒轮询，以毫秒精度渲染校正后的时间。所有处理在本地完成，不依赖云端服务。

## 精度等级

| 等级 | 偏移标准差 |
|-------|-----------|
| S+ | < 2ms |
| S | < 5ms |
| S- | < 10ms |
| A | < 30ms |
| B | < 50ms |
| C | < 100ms |
| D | ≥ 100ms |

## CI/CD 自动编译

推送标签触发全平台编译：

```bash
git tag v2.1.8
git push origin v2.1.8
```

GitHub Actions 自动编译：Windows MSI + NSIS、macOS DMG（x86_64 + aarch64）、Linux deb + AppImage。推送标签时还会自动更新 GitHub Release 附件和 `updater.json`。

---

MIT License
