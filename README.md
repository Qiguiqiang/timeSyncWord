# TimeSyncWord

简约的浏览器端高精度时间同步工具。通过 NTP 多服务器加权同步，在浏览器中显示校准后的精确时间。

## 界面

```
┌─────────────────────────────────────────┐
│           TimeSyncWord                   │
│               ● SYNCED ±0.5ms            │
│                                         │
│            12:34:56.789                  │
│   UTC: 2026-06-04 12:34:56.789          │
│   OFFSET: +0.52ms  |  RTT: 12ms         │
│                                         │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐   │
│  │  精度  │ │  偏移  │ │  延迟  │ │  采样  │   │
│  │  S+   │ │ -0.52│ │  12  │ │  18   │   │
│  │  tier │ │  ms  │ │  ms  │ │samples│   │
│  └──────┘ └──────┘ └──────┘ └──────┘   │
└─────────────────────────────────────────┘
```

- Cyberpunk/HUD 暗色主题，Orbitron 字体大数字显示
- 实时显示当前时间（HH:MM:SS.ms）、UTC、偏移量和 RTT
- 底部四张指标卡：精度评级（S+/S/A/B/C）、偏移、延迟、采样数
- 彩色状态指示：绿色（稳定）、黄（波动）、红（偏差过大）

## 快速开始

```bash
# 安装依赖
npm install

# 启动
npm start

# 浏览器打开
# http://localhost:13013
```

## 项目结构

```
TimeSyncWord/
├── server/
│   ├── index.js           # 服务器入口（HTTP + WebSocket）
│   ├── config.js          # 端口、NTP 服务器、同步参数
│   ├── time-service.js    # NTP 多服务器加权同步
│   └── signaling.js       # WebSocket 时间查询处理
├── public/
│   ├── index.html         # 主页面
│   ├── css/style.css      # 暗色 Cyberpunk 主题
│   └── js/app.js          # 客户端同步 + 显示逻辑
├── Dockerfile             # Docker 构建
├── docker-compose.yml     # Docker 编排
└── README.md
```

## 同步原理

1. 服务端每 5s 向多个 NTP 服务器（ntp.aliyun.com、ntp.tencent.com 等）发起查询
2. 每个服务器多次采样，按 RTT 排序剔除最高 10% 的异常值
3. 对剩余样本按 RTT 权重加权平均，计算系统时钟偏移
4. 客户端每 500ms 向服务端发起 WebSocket 时间查询
5. 客户端收到服务端时间戳后，同样按 RTT 过滤 + 加权平均计算出本地偏移
6. 本地时间 = Date.now() + offset，实时更新显示器

## Docker 部署

```bash
docker-compose up -d
```

## 配置

编辑 `server/config.js`：
- `port`：HTTP 端口（默认 13013）
- `ntpServers`：NTP 服务器列表
- `sync.samplesPerServer`：每服务器采样数
- `sync.resyncInterval`：重新同步间隔（ms）

## License

MIT
