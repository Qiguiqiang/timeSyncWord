const WebSocket = require('ws');
const timeService = require('./time-service');

function handleConnection(wss) {
  wss.on('connection', (ws) => {
    ws.on('message', (data) => {
      try {
        const msg = JSON.parse(data);
        if (msg.type === 'getTime') {
          ws.send(JSON.stringify({
            type: 'time',
            t1: msg.t1,
            t2: Date.now(),
            serverTime: timeService.getServerTimeMs(),
            t3: Date.now()
          }));
        }
      } catch (e) {}
    });

    ws.on('error', () => {});
  });
}

module.exports = { handleConnection };
