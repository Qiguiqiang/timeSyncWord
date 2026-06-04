const dgram = require('dgram');
const { ntpServers, sync } = require('./config');

class TimeService {
  constructor() {
    this.serverTime = BigInt(0);
    this.ntpOffsets = [];
    this.isSynced = false;
    this.lastSyncTime = 0;
  }

  // Get current server time in milliseconds
  getServerTimeMs() {
    return Date.now();
  }

  // Query a single NTP server
  async queryNTP(server) {
    return new Promise((resolve, reject) => {
      const client = dgram.createSocket('udp4');
      const timeout = setTimeout(() => {
        client.close();
        reject(new Error(`NTP timeout: ${server}`));
      }, 2000);

      // NTP packet (48 bytes)
      const packet = Buffer.alloc(48);
      packet[0] = 0x1b; // LI=0, VN=3, Mode=3 (client)

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

        // Parse NTP response - receive timestamp (bytes 32-39) and transmit timestamp (bytes 40-47)
        const T2 = this.parseNTPTimestamp(msg, 32);
        const T3 = this.parseNTPTimestamp(msg, 40);

        // Calculate offset: offset = ((T2-T1) + (T3-T4)) / 2
        const offset = ((T2 - T1) + (T3 - T4)) / 2;
        // Calculate RTT: RTT = (T4-T1) - (T3-T2)
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

  // Parse NTP timestamp from packet
  // NTP timestamp: 32-bit seconds since 1900 + 32-bit fractions
  parseNTPTimestamp(packet, offset) {
    const seconds = packet.readUInt32BE(offset);
    const fractions = packet.readUInt32BE(offset + 4);
    
    // Convert to milliseconds
    // NTP epoch is 1900-01-01, Unix epoch is 1970-01-01
    // Difference is 70 years in seconds = 2208988800
    const ntpEpoch = 2208988800;
    const unixSeconds = seconds - ntpEpoch;
    const milliseconds = unixSeconds * 1000 + (fractions * 1000) / 0x100000000;
    
    return milliseconds;
  }

  // Remove outliers from samples
  removeOutliers(samples, threshold) {
    if (samples.length < 3) return samples;
    
    const sorted = [...samples].sort((a, b) => a.rtt - b.rtt);
    const cutoff = Math.floor(samples.length * threshold);
    
    return sorted.slice(cutoff, sorted.length - cutoff);
  }

  // Calculate weighted average offset
  weightedAverage(samples) {
    if (samples.length === 0) return 0;
    
    let totalWeight = 0;
    let weightedSum = 0;
    
    for (const sample of samples) {
      const weight = 1 / (sample.rtt + 1); // Higher weight for lower RTT
      totalWeight += weight;
      weightedSum += sample.offset * weight;
    }
    
    return weightedSum / totalWeight;
  }

  // Sync with all NTP servers
  async syncWithNTP() {
    const allSamples = [];
    
    for (const server of ntpServers) {
      try {
        // Query each server multiple times
        for (let i = 0; i < sync.samplesPerServer; i++) {
          const sample = await this.queryNTP(server);
          if (Math.abs(sample.rtt) < sync.maxRTT) {
            allSamples.push(sample);
          }
        }
      } catch (err) {
        // Skip failed servers silently
      }
    }

    if (allSamples.length === 0) {
      return false;
    }

    // Remove outliers
    const filtered = this.removeOutliers(allSamples, sync.outlierThreshold);
    
    // Calculate final offset (in milliseconds)
    this.serverTime = this.weightedAverage(filtered);
    this.ntpOffsets = filtered.map(s => s.offset);
    this.isSynced = true;
    this.lastSyncTime = Date.now();

    console.log(`NTP synced: ${filtered.length} samples, offset: ${this.serverTime.toFixed(2)}ms`);
    return true;
  }

  // Get sync status for dashboard
  getSyncStatus() {
    return {
      isSynced: this.isSynced,
      lastSyncTime: this.lastSyncTime,
      serverTimeMs: this.getServerTimeMs(),
      sampleCount: this.ntpOffsets.length,
      avgOffset: this.ntpOffsets.length > 0 
        ? this.ntpOffsets.reduce((a, b) => a + b, 0) / this.ntpOffsets.length 
        : 0
    };
  }
}

module.exports = new TimeService();
