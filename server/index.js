const express = require('express');
const http = require('http');
const https = require('https');
const fs = require('fs');
const path = require('path');
const WebSocket = require('ws');
const os = require('os');
const config = require('./config');
const timeService = require('./time-service');
const { handleConnection } = require('./signaling');

const app = express();
app.use(express.static(path.join(__dirname, '..', 'public')));

const httpServer = http.createServer(app);
const wss = new WebSocket.Server({ server: httpServer });
handleConnection(wss);

if (config.ssl.enabled) {
  try {
    const opts = {
      key: fs.readFileSync(path.resolve(config.ssl.key)),
      cert: fs.readFileSync(path.resolve(config.ssl.cert))
    };
    const httpsServer = https.createServer(opts, app);
    const wss2 = new WebSocket.Server({ server: httpsServer });
    handleConnection(wss2);
    httpsServer.listen(config.sslPort, '0.0.0.0', () => console.log(`HTTPS on ${config.sslPort}`));
  } catch (e) { console.error('SSL error:', e.message); }
}

async function startNTP() {
  console.log('Syncing NTP...');
  await timeService.syncWithNTP();
  setInterval(() => timeService.syncWithNTP(), 5000);
}

httpServer.listen(config.port, '0.0.0.0', async () => {
  const ifaces = os.networkInterfaces();
  console.log(`\nTimeSyncWord`);
  console.log(`──────────────────`);
  console.log(`http://localhost:${config.port}`);
  for (const name of Object.keys(ifaces)) {
    for (const iface of ifaces[name]) {
      if (iface.family === 'IPv4' && !iface.internal) {
        console.log(`http://${iface.address}:${config.port}`);
      }
    }
  }
  console.log(`──────────────────`);
  await startNTP();
});

process.on('SIGINT', () => process.exit(0));
process.on('SIGTERM', () => process.exit(0));
