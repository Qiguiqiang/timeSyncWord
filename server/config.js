module.exports = {
  port: parseInt(process.env.PORT) || 13013,
  sslPort: parseInt(process.env.SSL_PORT) || 13014,

  ssl: {
    enabled: process.env.SSL_ENABLED === 'true',
    key: process.env.SSL_KEY_PATH || './certs/server.key',
    cert: process.env.SSL_CERT_PATH || './certs/server.crt'
  },

  ntpServers: [
    { host: 'ntp.tencent.com', name: 'Tencent', label: '腾讯云' },
    { host: 'ntp.aliyun.com', name: 'Aliyun', label: '阿里云' },
    { host: 'time.asia.apple.com', name: 'Apple', label: 'Apple Asia' },
    { host: 'time.google.com', name: 'Google', label: 'Google' },
    { host: 'pool.ntp.org', name: 'Pool', label: 'pool.ntp.org' }
  ],

  defaultServer: 'ntp.tencent.com',

  sync: {
    samplesPerServer: 5,
    outlierThreshold: 0.1,
    resyncInterval: 5000,
    maxRTT: 500
  },

  ws: {
    heartbeatInterval: 10000,
    maxConnections: 1000
  }
};
