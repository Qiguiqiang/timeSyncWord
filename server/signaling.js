const WebSocket = require('ws');
const timeService = require('./time-service');
const { ntpServers } = require('./config');

let broadcastInterval = null;

function handleConnection(wss) {
  broadcastInterval = setInterval(() => {
    if (wss.clients.size === 0) return;

    const status = timeService.getSyncStatus();
    const timeMsg = JSON.stringify({
      type: 'time',
      serverTime: timeService.getServerTimeMs(),
      t2: Date.now(),
      ntpServer: status.activeServer,
      ntpRtt: status.activeRtt,
      ntpOffset: status.avgOffset,
      serverLatencies: status.serverLatencies
    });

    wss.clients.forEach(client => {
      if (client.readyState === WebSocket.OPEN) {
        client.send(timeMsg);
      }
    });
  }, 2000);

  wss.on('connection', (ws) => {
    const status = timeService.getSyncStatus();
    ws.send(JSON.stringify({
      type: 'time',
      serverTime: timeService.getServerTimeMs(),
      t2: Date.now(),
      ntpServer: status.activeServer,
      ntpRtt: status.activeRtt,
      ntpOffset: status.avgOffset,
      serverLatencies: status.serverLatencies
    }));

    ws.on('message', (data) => {
      try {
        const msg = JSON.parse(data);
        if (msg.type === 'getTime') {
          ws.send(JSON.stringify({
            type: 'timeResponse',
            t1: msg.t1,
            serverTime: timeService.getServerTimeMs()
          }));
        }
        if (msg.type === 'setNtpServer') {
          const changed = timeService.setActiveServer(msg.server);
          if (changed) {
            timeService.syncWithNTP();
            const status = timeService.getSyncStatus();
            ws.send(JSON.stringify({
              type: 'ntpServerChanged',
              ntpServer: status.activeServer,
              ntpRtt: status.activeRtt,
              serverLatencies: status.serverLatencies
            }));
          }
        }
      } catch (e) {}
    });

    ws.on('error', () => {});
  });

  wss.on('close', () => {
    if (broadcastInterval) {
      clearInterval(broadcastInterval);
    }
  });
}

module.exports = { handleConnection };
