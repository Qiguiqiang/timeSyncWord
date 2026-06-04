module.exports = {
  port: parseInt(process.env.PORT) || 13013,
  sslPort: parseInt(process.env.SSL_PORT) || 13014,

  ssl: {
    enabled: process.env.SSL_ENABLED === 'true',
    key: process.env.SSL_KEY_PATH || './certs/server.key',
    cert: process.env.SSL_CERT_PATH || './certs/server.crt'
  },

  ntpServers: [
    'ntp.aliyun.com',
    'ntp.tencent.com',
    'time.asia.apple.com',
    'time.google.com',
    'pool.ntp.org'
  ],

  sync: {
    samplesPerServer: 10,
    outlierThreshold: 0.1,
    resyncInterval: 5000,
    maxRTT: 500
  },

  ws: {
    heartbeatInterval: 10000,
    maxConnections: 1000
  }
};
