const dgram = require('dgram');
const { ntpServers, sync, defaultServer } = require('./config');

class TimeService {
  constructor() {
    this.serverTime = BigInt(0);
    this.ntpOffsets = [];
    this.isSynced = false;
    this.lastSyncTime = 0;
    this.activeServer = defaultServer;
    this.serverLatencies = {};
    ntpServers.forEach(s => { this.serverLatencies[s.host] = { rtt: -1, status: 'unknown' }; });
  }

  getServerTimeMs() {
    return Date.now();
  }

  setActiveServer(host) {
    if (ntpServers.some(s => s.host === host)) {
      this.activeServer = host;
      this.isSynced = false;
      this.ntpOffsets = [];
      return true;
    }
    return false;
  }

  async queryNTP(server) {
    return new Promise((resolve, reject) => {
      const client = dgram.createSocket('udp4');
      const timeout = setTimeout(() => {
        client.close();
        reject(new Error(`NTP timeout: ${server}`));
      }, 2000);

      const packet = Buffer.alloc(48);
      packet[0] = 0x1b;

      const T1 = Date.now();

      client.send(packet, 123, server, (err) => {
        if (err) {
          clearTimeout(timeout);
          client.close();
          reject(err);
        }
      });

      client.on('message', (msg) => {
        clearTimeout(timeout);
        const T4 = Date.now();

        const T2 = this.parseNTPTimestamp(msg, 32);
        const T3 = this.parseNTPTimestamp(msg, 40);

        const offset = ((T2 - T1) + (T3 - T4)) / 2;
        const rtt = (T4 - T1) - (T3 - T2);

        client.close();
        resolve({ server, offset, rtt });
      });

      client.on('error', (err) => {
        clearTimeout(timeout);
        client.close();
        reject(err);
      });
    });
  }

  parseNTPTimestamp(packet, offset) {
    const seconds = packet.readUInt32BE(offset);
    const fractions = packet.readUInt32BE(offset + 4);
    const ntpEpoch = 2208988800;
    const unixSeconds = seconds - ntpEpoch;
    const milliseconds = unixSeconds * 1000 + (fractions * 1000) / 0x100000000;
    return milliseconds;
  }

  removeOutliers(samples, threshold) {
    if (samples.length < 3) return samples;
    const sorted = [...samples].sort((a, b) => a.rtt - b.rtt);
    const cutoff = Math.floor(samples.length * threshold);
    return sorted.slice(cutoff, samples.length - cutoff);
  }

  weightedAverage(samples) {
    if (samples.length === 0) return 0;
    let totalWeight = 0;
    let weightedSum = 0;
    for (const sample of samples) {
      const weight = 1 / (sample.rtt + 1);
      totalWeight += weight;
      weightedSum += sample.offset * weight;
    }
    return weightedSum / totalWeight;
  }

  async syncWithNTP() {
    const allSamples = [];

    for (const ntp of ntpServers) {
      try {
        for (let i = 0; i < 1; i++) {
          const sample = await this.queryNTP(ntp.host);
          if (Math.abs(sample.rtt) < sync.maxRTT) {
            allSamples.push(sample);
          }
        }
        const avgRtt = allSamples.filter(s => s.server === ntp.host).reduce((sum, s, _, arr) => sum + s.rtt / arr.length, 0);
        this.serverLatencies[ntp.host] = { rtt: Math.round(avgRtt), status: 'ok' };
      } catch (err) {
        this.serverLatencies[ntp.host] = { rtt: -1, status: 'timeout' };
      }
    }

    const activeSamples = allSamples.filter(s => s.server === this.activeServer);
    if (activeSamples.length === 0) {
      return false;
    }

    const filtered = this.removeOutliers(activeSamples, sync.outlierThreshold);
    this.serverTime = this.weightedAverage(filtered);
    this.ntpOffsets = filtered.map(s => s.offset);
    this.isSynced = true;
    this.lastSyncTime = Date.now();

    return true;
  }

  getSyncStatus() {
    const activeLatency = this.serverLatencies[this.activeServer] || { rtt: -1 };
    return {
      isSynced: this.isSynced,
      lastSyncTime: this.lastSyncTime,
      serverTimeMs: this.getServerTimeMs(),
      sampleCount: this.ntpOffsets.length,
      avgOffset: this.ntpOffsets.length > 0 
        ? this.ntpOffsets.reduce((a, b) => a + b, 0) / this.ntpOffsets.length 
        : 0,
      activeServer: this.activeServer,
      activeRtt: activeLatency.rtt,
      serverLatencies: this.serverLatencies
    };
  }
}

module.exports = new TimeService();
