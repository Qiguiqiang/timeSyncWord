use serde::Serialize;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
pub struct NtpSample {
    pub server: String,
    pub offset: f64,
    pub rtt: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerLatency {
    pub rtt: i64,
    pub status: String,
}

pub fn query_ntp(server: &str) -> Result<NtpSample, String> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| format!("bind error: {}", e))?;
    socket.set_read_timeout(Some(Duration::from_millis(2000)))
        .map_err(|e| format!("set timeout error: {}", e))?;

    let mut packet = [0u8; 48];
    packet[0] = 0x1b;

    let t1 = wall_time_ms();

    socket
        .send_to(&packet, format!("{}:123", server))
        .map_err(|e| format!("send error: {}", e))?;

    let mut buf = [0u8; 48];
    let n = socket.recv(&mut buf).map_err(|e| format!("recv error: {}", e))?;
    if n < 48 {
        return Err(format!("short response: {} bytes", n));
    }

    let t4 = wall_time_ms();

    let t2 = parse_ntp_timestamp(&buf, 32);
    let t3 = parse_ntp_timestamp(&buf, 40);

    let now = t4 as f64;
    if t2 < 946684800000.0 || t2 > now + 5000.0 || t3 < 946684800000.0 || t3 > now + 5000.0 {
        return Err(format!("invalid ntp timestamp from {}", server));
    }

    let offset = ((t2 - t1 as f64) + (t3 - t4 as f64)) / 2.0;
    let rtt = (t4 as f64 - t1 as f64) - (t3 - t2);

    Ok(NtpSample {
        server: server.to_string(),
        offset,
        rtt: rtt.max(0.0),
    })
}

fn parse_ntp_timestamp(buf: &[u8], offset: usize) -> f64 {
    let secs = u32::from_be_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ]);
    let frac = u32::from_be_bytes([
        buf[offset + 4],
        buf[offset + 5],
        buf[offset + 6],
        buf[offset + 7],
    ]);
    let unix_secs = secs.saturating_sub(2_208_988_800);
    (unix_secs as f64) * 1000.0 + (frac as f64) * 1000.0 / 4_294_967_296.0
}

fn wall_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn weighted_average(samples: &[NtpSample]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let total_weight: f64 = samples.iter().map(|s| 1.0 / (s.rtt + 1.0)).sum();
    let weighted_sum: f64 = samples
        .iter()
        .map(|s| s.offset * (1.0 / (s.rtt + 1.0)))
        .sum();
    weighted_sum / total_weight
}

pub fn remove_outliers(samples: &[NtpSample], threshold: f64) -> Vec<NtpSample> {
    if samples.len() < 3 {
        return samples.to_vec();
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.rtt.partial_cmp(&b.rtt).unwrap_or(std::cmp::Ordering::Equal));
    let cutoff = (samples.len() as f64 * threshold) as usize;
    let end = samples.len() - cutoff;
    if cutoff >= end {
        return samples.to_vec();
    }
    sorted[cutoff..end].to_vec()
}
