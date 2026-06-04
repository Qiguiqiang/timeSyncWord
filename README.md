# OpenTimeSync

Browser-based high-precision time synchronization tool via NTP-weighted clock.

## Features

- **NTP Multi-Server Sync** - Queries 5 NTP servers with weighted averaging and outlier filtering
- **Server Broadcast Mode** - Server pushes time to all clients every 2 seconds (low load)
- **Periodic RTT Measurement** - Network latency measured every 30 seconds
- **Precision Grading** - S+/S/S-/A/B/C/D grades based on offset stability
- **Timezone Support** - 14 timezones with localStorage persistence
- **Cyberpunk UI** - Dark theme with neon glow effects, responsive design
- **Docker Ready** - Multi-stage Docker build with docker-compose

## Architecture

```
NTP Servers (UDP) → Server (Node.js) → WebSocket Broadcast → Client (Browser)
                                                        ↓
                                              Client RTT Ping (every 30s)
```

- **Server**: Queries NTP servers, calculates accurate UTC time, broadcasts to clients
- **Client**: Receives server time, calculates offset, displays synchronized time
- **Hybrid Mode**: Broadcast for time sync + periodic ping for RTT measurement

## Quick Start

```bash
# Install dependencies
npm install

# Start server
npm start

# Open browser
# http://localhost:13013
```

## Project Structure

```
OpenTimeSync/
├── server/
│   ├── index.js           # Server entry (HTTP + WebSocket)
│   ├── config.js          # NTP servers, sync parameters
│   ├── time-service.js    # NTP multi-server weighted sync
│   └── signaling.js       # WebSocket broadcast + RTT handler
├── public/
│   ├── index.html         # Main page
│   ├── css/style.css      # Cyberpunk dark theme
│   └── js/app.js          # Client sync + timezone + UI
├── electron/
│   ├── main.js            # Electron main process
│   └── preload.js         # Preload script
├── Dockerfile             # Multi-stage Docker build
├── docker-compose.yml     # Docker orchestration
└── README.md
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

## Docker Deployment

```bash
# Build and run
docker-compose up -d

# View logs
docker logs -f opentimesync

# Stop
docker-compose down
```

## Configuration

Edit `server/config.js`:
- `port`: HTTP port (default 13013)
- `ntpServers`: NTP server list
- `sync.samplesPerServer`: Samples per NTP server
- `sync.resyncInterval`: NTP resync interval (ms)

## Time Synchronization Flow

1. **Server → NTP**: Server queries 5 NTP servers (10 samples each)
2. **Server → Client**: Server broadcasts time every 2 seconds via WebSocket
3. **Client Calculation**: Client calculates offset = serverTime - localTime
4. **Client → Server**: Client pings server every 30 seconds for RTT measurement
5. **Display**: Client shows synchronized time with precision grade

## License

MIT