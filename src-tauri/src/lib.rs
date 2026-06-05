mod ntp;

use ntp::{query_ntp, remove_outliers, weighted_average, NtpSample, ServerLatency};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Emitter;

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
    last_payload: Mutex<Option<NtpTimePayload>>,
    cycle_count: Mutex<u64>,
    auto_sync: Mutex<bool>,
    last_sync_cycle: Mutex<u64>,
    sync_interval_secs: Mutex<u64>,
}

#[allow(non_snake_case)]
#[repr(C)]
struct SYSTEMTIME {
    wYear: u16, wMonth: u16, wDayOfWeek: u16,
    wDay: u16, wHour: u16, wMinute: u16,
    wSecond: u16, wMilliseconds: u16,
}

#[cfg(windows)]
extern "system" {
    fn SetSystemTime(lpSystemTime: *const SYSTEMTIME) -> i32;
    fn GetCurrentProcess() -> isize;
    fn OpenProcessToken(hProcess: isize, dwDesiredAccess: u32, phToken: &mut isize) -> i32;
    fn CloseHandle(hObject: isize) -> i32;
    fn LookupPrivilegeValueW(lpSystemName: *const u16, lpName: *const u16, lpLuid: &mut i64) -> i32;
    fn AdjustTokenPrivileges(
        hToken: isize, bDisableAll: i32, lpNewState: *const TOKEN_PRIVILEGES,
        cbBuffer: u32, lpPreviousState: *mut TOKEN_PRIVILEGES, cbReturn: &mut u32,
    ) -> i32;
}

#[allow(non_snake_case)]
#[repr(C)]
struct LUID_AND_ATTRIBUTES { luid: i64, attributes: u32 }

#[allow(non_snake_case)]
#[repr(C)]
struct TOKEN_PRIVILEGES {
    privilege_count: u32,
    privileges: [LUID_AND_ATTRIBUTES; 1],
}

const SE_SYSTEMTIME_NAME: &str = "SeSystemtimePrivilege\0";
const TOKEN_QUERY: u32 = 0x0008;
const TOKEN_ADJUST_PRIVILEGES: u32 = 0x0020;
const SE_PRIVILEGE_ENABLED: u32 = 0x00000002;

#[cfg(windows)]
fn enable_privilege() -> Result<(), String> {
    unsafe {
        let h = GetCurrentProcess();
        let mut token: isize = 0;
        if OpenProcessToken(h, TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token) == 0 {
            return Err("无法打开进程令牌".to_string());
        }
        let mut luid: i64 = 0;
        let name = SE_SYSTEMTIME_NAME.encode_utf16().collect::<Vec<_>>();
        if LookupPrivilegeValueW(std::ptr::null(), name.as_ptr(), &mut luid) == 0 {
            CloseHandle(token);
            return Err("无法查询特权".to_string());
        }
        let tp = TOKEN_PRIVILEGES {
            privilege_count: 1,
            privileges: [LUID_AND_ATTRIBUTES { luid, attributes: SE_PRIVILEGE_ENABLED }],
        };
        let mut prev: TOKEN_PRIVILEGES = std::mem::zeroed();
        let mut ret: u32 = 0;
        AdjustTokenPrivileges(
            token, 0, &tp,
            std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
            &mut prev, &mut ret,
        );
        CloseHandle(token);
        Ok(())
    }
}

#[cfg(windows)]
fn set_windows_system_time(server_time_ms: u64) -> Result<(), String> {
    let total_secs = server_time_ms / 1000;
    let millis = (server_time_ms % 1000) as u16;
    let seconds = (total_secs % 60) as u16;
    let minutes = ((total_secs / 60) % 60) as u16;
    let hours = ((total_secs / 3600) % 24) as u16;
    let days = total_secs / 86400;

    let mut y = 1970i64;
    let mut rem = days as i64;
    loop {
        let diy = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if rem < diy { break; }
        rem -= diy; y += 1;
    }
    let md: [i64; 12] = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
        [31,29,31,30,31,30,31,31,30,31,30,31]
    } else {
        [31,28,31,30,31,30,31,31,30,31,30,31]
    };
    let mut mon: u16 = 1;
    for &d in md.iter() {
        if rem < d { break; }
        rem -= d; mon += 1;
    }

    if let Err(e) = enable_privilege() {
        return Err(format!("权限不足: {}", e));
    }

    let st = SYSTEMTIME {
        wYear: y as u16, wMonth: mon, wDayOfWeek: 0, wDay: (rem + 1) as u16,
        wHour: hours, wMinute: minutes, wSecond: seconds, wMilliseconds: millis,
    };

    unsafe {
        if SetSystemTime(&st) == 0 {
            Err("设置系统时间失败，请以管理员身份运行此程序".to_string())
        } else {
            Ok(())
        }
    }
}

#[cfg(not(windows))]
fn set_windows_system_time(_server_time_ms: u64) -> Result<(), String> {
    Err("仅 Windows 支持系统时间同步".to_string())
}

#[tauri::command]
fn ping() -> String {
    "pong".to_string()
}

#[tauri::command]
fn sync_system_time(state: tauri::State<Arc<AppState>>) -> Result<String, String> {
    let payload = state.last_payload.lock().unwrap().clone();
    match payload {
        Some(p) => {
            set_windows_system_time(p.server_time)?;
            *state.last_sync_cycle.lock().unwrap() = *state.cycle_count.lock().unwrap();
            Ok("系统时间已同步".to_string())
        }
        None => Err("尚无 NTP 数据".to_string()),
    }
}

#[tauri::command]
fn set_auto_sync(state: tauri::State<Arc<AppState>>, enabled: bool) {
    *state.auto_sync.lock().unwrap() = enabled;
}

#[tauri::command]
fn get_auto_sync(state: tauri::State<Arc<AppState>>) -> bool {
    *state.auto_sync.lock().unwrap()
}

#[tauri::command]
fn set_sync_interval(state: tauri::State<Arc<AppState>>, seconds: u64) {
    *state.sync_interval_secs.lock().unwrap() = seconds.max(5).min(3600);
}

#[tauri::command]
fn get_sync_interval(state: tauri::State<Arc<AppState>>) -> u64 {
    *state.sync_interval_secs.lock().unwrap()
}

#[tauri::command]
fn set_ntp_server(state: tauri::State<Arc<AppState>>, server: String) -> bool {
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
fn get_ntp_status(state: tauri::State<Arc<AppState>>) -> Option<NtpTimePayload> {
    state.last_payload.lock().unwrap().clone()
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

#[tauri::command]
fn start_drag(window: tauri::Window) {
    let _ = window.start_dragging();
}

#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn run_ntp_loop(app_handle: tauri::AppHandle, app_state: Arc<AppState>) {
    std::thread::spawn(move || {
        loop {
            let active_host = app_state.active_server.lock().unwrap().host.clone();

            let mut server_latencies = HashMap::new();
            let mut active_samples: Vec<NtpSample> = Vec::new();

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

            let ntp_rtt = if active_samples.is_empty() {
                -1.0
            } else {
                active_samples.iter().map(|s| s.rtt).fold(f64::MAX, |a, b| a.min(b))
            };

            let corrected_time = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64 + ntp_offset) as u64;

            let payload = NtpTimePayload {
                server_time: corrected_time,
                ntp_offset,
                ntp_rtt: if ntp_rtt.is_finite() && ntp_rtt > 0.0 && ntp_rtt < 1_000_000.0 {
                    ntp_rtt.round() as i64
                } else {
                    -1
                },
                ntp_server: active_host,
                server_latencies,
            };

            let cycle = *app_state.cycle_count.lock().unwrap();
            *app_state.last_payload.lock().unwrap() = Some(payload.clone());
            *app_state.cycle_count.lock().unwrap() = cycle + 1;

            if *app_state.auto_sync.lock().unwrap() {
                let last_sync = *app_state.last_sync_cycle.lock().unwrap();
                let interval_secs = *app_state.sync_interval_secs.lock().unwrap();
                let cooldown = (interval_secs + 1) / 2;
                if cycle >= last_sync + cooldown {
                    if let Some(p) = app_state.last_payload.lock().unwrap().clone() {
                        let _ = set_windows_system_time(p.server_time);
                        *app_state.last_sync_cycle.lock().unwrap() = cycle;
                    }
                }
            }

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
        last_payload: Mutex::new(None),
        cycle_count: Mutex::new(0),
        auto_sync: Mutex::new(false),
        last_sync_cycle: Mutex::new(0),
        sync_interval_secs: Mutex::new(30),
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            ping,
            get_version,
            sync_system_time,
            set_auto_sync,
            get_auto_sync,
            set_sync_interval,
            get_sync_interval,
            set_ntp_server,
            get_ntp_status,
            minimize_window,
            maximize_window,
            close_window,
            start_drag,
        ])
        .setup(move |app| {
            run_ntp_loop(app.handle().clone(), app_state.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
