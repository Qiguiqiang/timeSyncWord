mod ntp;

use ntp::{query_ntp, remove_outliers, weighted_average, ServerLatency};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const NTP_SERVERS: &[(&str, &str, &str)] = &[
    ("ntp.tencent.com", "Tencent", "腾讯云"),
    ("ntp.aliyun.com", "Aliyun", "阿里云"),
    ("time.asia.apple.com", "Apple", "Apple Asia"),
    ("time.google.com", "Google", "Google"),
    ("pool.ntp.org", "Pool", "pool.ntp.org"),
];

#[derive(Debug, Clone, Serialize)]
struct NtpTimePayload {
    server_time: u64,
    ntp_offset: f64,
    ntp_rtt: i64,
    ntp_server: String,
    server_latencies: HashMap<String, ServerLatency>,
}

#[derive(Debug, Clone, Serialize)]
struct ActiveServer {
    host: String,
    name: String,
    label: String,
}

struct AppState {
    active_server: Mutex<ActiveServer>,
    running: AtomicBool,
}

#[tauri::command]
fn set_ntp_server(state: tauri::State<AppState>, server: String) -> bool {
    for s in NTP_SERVERS {
        if s.0 == server {
            let mut active = state.active_server.lock().unwrap();
            active.host = s.0.to_string();
            active.name = s.1.to_string();
            active.label = s.2.to_string();
            return true;
        }
    }
    false
}

#[tauri::command]
fn get_active_server(state: tauri::State<AppState>) -> ActiveServer {
    state.active_server.lock().unwrap().clone()
}

#[tauri::command]
fn minimize_window(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn maximize_window(window: tauri::Window) {
    if let Ok(true) = window.is_maximized() {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
fn close_window(window: tauri::Window) {
    let _ = window.close();
}

fn run_ntp_loop(app_handle: AppHandle, app_state: Arc<AppState>) {
    std::thread::spawn(move || {
        loop {
            if !app_state.running.load(Ordering::Relaxed) {
                break;
            }

            let active_host = app_state.active_server.lock().unwrap().host.clone();

            let mut server_latencies = HashMap::new();
            let mut active_samples = Vec::new();

            for (host, _name, _label) in NTP_SERVERS {
                match query_ntp(host) {
                    Ok(sample) => {
                        server_latencies.insert(
                            host.to_string(),
                            ServerLatency {
                                rtt: sample.rtt.round() as i64,
                                status: "ok".to_string(),
                            },
                        );
                        if *host == active_host {
                            active_samples.push(sample);
                        }
                    }
                    Err(_) => {
                        server_latencies.insert(
                            host.to_string(),
                            ServerLatency {
                                rtt: -1,
                                status: "timeout".to_string(),
                            },
                        );
                    }
                }
            }

            let ntp_offset = if !active_samples.is_empty() {
                let filtered = remove_outliers(&active_samples, 0.1);
                weighted_average(&filtered)
            } else {
                0.0
            };

            let ntp_rtt = active_samples
                .iter()
                .map(|s| s.rtt)
                .fold(f64::MAX, |a, b| a.min(b));

            let payload = NtpTimePayload {
                server_time: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                ntp_offset,
                ntp_rtt: if ntp_rtt.is_finite() && ntp_rtt > 0.0 {
                    ntp_rtt.round() as i64
                } else {
                    -1
                },
                ntp_server: active_host,
                server_latencies,
            };

            let _ = app_handle.emit("ntp-time", &payload);

            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(AppState {
        active_server: Mutex::new(ActiveServer {
            host: NTP_SERVERS[0].0.to_string(),
            name: NTP_SERVERS[0].1.to_string(),
            label: NTP_SERVERS[0].2.to_string(),
        }),
        running: AtomicBool::new(true),
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            set_ntp_server,
            get_active_server,
            minimize_window,
            maximize_window,
            close_window,
        ])
        .setup(move |app| {
            run_ntp_loop(app.handle().clone(), app_state.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
