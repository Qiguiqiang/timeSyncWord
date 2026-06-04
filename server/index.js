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

function setupApp() {
  const app = express();
  app.use(express.static(path.join(__dirname, '..', 'public')));
  return app;
}

function setupWSS(httpServer) {
  const wss = new WebSocket.Server({ server: httpServer });
  handleConnection(wss);
}

function setupSSL(app) {
  if (config.ssl.enabled) {
    try {
      const opts = {
        key: fs.readFileSync(path.resolve(config.ssl.key)),
        cert: fs.readFileSync(path.resolve(config.ssl.cert))
      };
      const httpsServer = https.createServer(opts, app);
      setupWSS(httpsServer);
      httpsServer.listen(config.sslPort, '0.0.0.0', () => console.log(`HTTPS on ${config.sslPort}`));
    } catch (e) { console.error('SSL error:', e.message); }
  }
}

async function startNTP() {
  try {
    console.log('Syncing NTP...');
    await timeService.syncWithNTP();
    setInterval(() => timeService.syncWithNTP().catch(() => {}), 5000);
  } catch (e) {
    console.error('NTP initial sync failed, retrying in 5s...');
    setTimeout(() => startNTP(), 5000);
  }
}

async function createServer() {
  const app = setupApp();
  const httpServer = http.createServer(app);
  setupWSS(httpServer);
  setupSSL(app);

  return new Promise((resolve) => {
    httpServer.listen({ port: config.port, exclusive: false }, () => {
      const ifaces = os.networkInterfaces();
      console.log(`\nOpenTimeSync`);
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
      resolve({ port: config.port, close: () => httpServer.close() });
      startNTP();
    });
  });
}

if (require.main === module) {
  createServer().catch(console.error);
  process.on('SIGINT', () => process.exit(0));
  process.on('SIGTERM', () => process.exit(0));
}

module.exports = { createServer };
