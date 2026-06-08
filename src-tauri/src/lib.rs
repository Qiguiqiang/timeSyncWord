mod ntp;
use ntp::{query_ntp, remove_outliers, weighted_average, NtpSample, ServerLatency};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    utils::config::Color,
    AppHandle, Emitter, Manager, PhysicalPosition, State, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, Window, WindowEvent,
};
use tauri_plugin_updater::UpdaterExt;

const NTP_SERVERS: &[(&str, &str, &str)] = &[
    ("ntp.tencent.com", "Tencent", "腾讯云"),
    ("ntp.aliyun.com", "Aliyun", "阿里云"),
    ("time.asia.apple.com", "Apple", "Apple Asia"),
    ("time.google.com", "Google", "Google"),
    ("pool.ntp.org", "Pool", "pool.ntp.org"),
];

const WIDGET_LABEL: &str = "widget";
const MAIN_LABEL: &str = "main";
const SPLASH_LABEL: &str = "splashscreen";
const TRAY_ID: &str = "system-tray";
const TRAY_SHOW_ID: &str = "tray-show";
const TRAY_WIDGET_ID: &str = "tray-widget";
const TRAY_EXIT_ID: &str = "tray-exit";
const MASTER_PORT: u16 = 36363;
const CALIBRATION_INTERVAL_SECS: u64 = 2;
const CALIBRATION_TIMEOUT_SECS: u64 = 30;
const SPLASH_MIN_VISIBLE_MS: u64 = 1200;
const SPLASH_MAX_VISIBLE_MS: u64 = 3200;
const WIDGET_WIDTH: f64 = 172.0;
const WIDGET_HEIGHT: f64 = 34.0;
const WIDGET_DEFAULT_RIGHT_MARGIN: i32 = 28;
const WIDGET_DEFAULT_BOTTOM_MARGIN: i32 = 16;
const WIDGET_MIN_SCALE: u32 = 80;
const WIDGET_MAX_SCALE: u32 = 220;

fn default_widget_scale() -> u32 {
    100
}

fn clamp_widget_scale(scale: u32) -> u32 {
    scale.clamp(WIDGET_MIN_SCALE, WIDGET_MAX_SCALE)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NtpTimePayload {
    server_time: u64,
    ntp_offset: f64,
    ntp_rtt: i64,
    has_fresh_data: bool,
    sample_id: u64,
    ntp_server: String,
    server_latencies: HashMap<String, ServerLatency>,
    sync_mode: SyncMode,
    source_label: String,
    calibration_stage: CalibrationStage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActiveServer {
    host: String,
    name: String,
    label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum SyncMode {
    LocalNtp,
    Master,
    Slave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum CalibrationStage {
    Idle,
    Calibrating,
    Stable,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedConfig {
    auto_sync: bool,
    sync_interval_secs: u64,
    ntp_server: String,
    sync_mode: SyncMode,
    master_host: String,
    pair_code: String,
    widget_enabled: bool,
    widget_x: Option<i32>,
    widget_y: Option<i32>,
    #[serde(default = "default_widget_scale")]
    widget_scale: u32,
}

impl Default for SavedConfig {
    fn default() -> Self {
        Self {
            auto_sync: true,
            sync_interval_secs: 5,
            ntp_server: NTP_SERVERS[0].0.to_string(),
            sync_mode: SyncMode::LocalNtp,
            master_host: format!("127.0.0.1:{MASTER_PORT}"),
            pair_code: default_pair_code(),
            widget_enabled: false,
            widget_x: None,
            widget_y: None,
            widget_scale: default_widget_scale(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncSettingsPayload {
    auto_sync: bool,
    sync_interval_secs: u64,
    ntp_server: String,
    sync_mode: SyncMode,
    master_host: String,
    pair_code: String,
    widget_enabled: bool,
    widget_scale: u32,
    calibration_stage: CalibrationStage,
    active_servers: Vec<ActiveServer>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeContextPayload {
    is_widget: bool,
    version: String,
}

struct AppState {
    active_server: Mutex<ActiveServer>,
    last_payload: Mutex<Option<NtpTimePayload>>,
    cycle_count: Mutex<u64>,
    auto_sync: Mutex<bool>,
    sync_interval_secs: Mutex<u64>,
    sync_mode: Mutex<SyncMode>,
    master_host: Mutex<String>,
    pair_code: Mutex<String>,
    widget_enabled: Mutex<bool>,
    collapsed_to_tray: Mutex<bool>,
    widget_dismissed: Mutex<bool>,
    widget_force_visible: Mutex<bool>,
    widget_position: Mutex<Option<(i32, i32)>>,
    widget_scale: Mutex<u32>,
    calibration_stage: Mutex<CalibrationStage>,
    calibration_started_at: Mutex<Option<Instant>>,
    calibration_sample_count: Mutex<u32>,
    startup_completed: Mutex<bool>,
    config_path: Mutex<Option<PathBuf>>,
    update_status: Mutex<UpdateStatusPayload>,
    downloaded_update: Mutex<Option<DownloadedUpdate>>,
}

#[derive(Clone)]
struct DownloadedUpdate {
    update: tauri_plugin_updater::Update,
    bytes: Vec<u8>,
    notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateStatusPayload {
    phase: String,
    current_version: String,
    version: Option<String>,
    message: String,
    notes: Option<String>,
    downloaded_bytes: Option<u64>,
    total_bytes: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MasterTimeRequest {
    pair_code: String,
    client_sent_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MasterTimeResponse {
    ok: bool,
    message: String,
    payload: Option<NtpTimePayload>,
}

#[allow(non_snake_case)]
#[repr(C)]
struct SYSTEMTIME {
    wYear: u16,
    wMonth: u16,
    wDayOfWeek: u16,
    wDay: u16,
    wHour: u16,
    wMinute: u16,
    wSecond: u16,
    wMilliseconds: u16,
}

#[cfg(windows)]
extern "system" {
    fn SetSystemTime(lpSystemTime: *const SYSTEMTIME) -> i32;
    fn GetCurrentProcess() -> isize;
    fn OpenProcessToken(hProcess: isize, dwDesiredAccess: u32, phToken: &mut isize) -> i32;
    fn CloseHandle(hObject: isize) -> i32;
    fn LookupPrivilegeValueW(
        lpSystemName: *const u16,
        lpName: *const u16,
        lpLuid: &mut i64,
    ) -> i32;
    fn AdjustTokenPrivileges(
        hToken: isize,
        bDisableAll: i32,
        lpNewState: *const TOKEN_PRIVILEGES,
        cbBuffer: u32,
        lpPreviousState: *mut TOKEN_PRIVILEGES,
        cbReturn: &mut u32,
    ) -> i32;
}

#[allow(non_snake_case)]
#[repr(C)]
struct LUID_AND_ATTRIBUTES {
    luid: i64,
    attributes: u32,
}

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

impl CalibrationStage {
    fn is_calibrating(self) -> bool {
        matches!(self, Self::Calibrating)
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn default_pair_code() -> String {
    format!("{:06}", now_unix_ms() % 1_000_000)
}

fn available_servers() -> Vec<ActiveServer> {
    NTP_SERVERS
        .iter()
        .map(|(host, name, label)| ActiveServer {
            host: (*host).to_string(),
            name: (*name).to_string(),
            label: (*label).to_string(),
        })
        .collect()
}

fn find_server(server: &str) -> ActiveServer {
    NTP_SERVERS
        .iter()
        .find(|(host, _, _)| *host == server)
        .map(|(host, name, label)| ActiveServer {
            host: (*host).to_string(),
            name: (*name).to_string(),
            label: (*label).to_string(),
        })
        .unwrap_or_else(|| ActiveServer {
            host: NTP_SERVERS[0].0.to_string(),
            name: NTP_SERVERS[0].1.to_string(),
            label: NTP_SERVERS[0].2.to_string(),
        })
}

fn current_saved_config(state: &Arc<AppState>) -> SavedConfig {
    let widget_position = *state.widget_position.lock().unwrap();
    SavedConfig {
        auto_sync: *state.auto_sync.lock().unwrap(),
        sync_interval_secs: *state.sync_interval_secs.lock().unwrap(),
        ntp_server: state.active_server.lock().unwrap().host.clone(),
        sync_mode: *state.sync_mode.lock().unwrap(),
        master_host: state.master_host.lock().unwrap().clone(),
        pair_code: state.pair_code.lock().unwrap().clone(),
        widget_enabled: *state.widget_enabled.lock().unwrap(),
        widget_x: widget_position.map(|position| position.0),
        widget_y: widget_position.map(|position| position.1),
        widget_scale: *state.widget_scale.lock().unwrap(),
    }
}

fn persist_config(state: &Arc<AppState>) -> Result<(), String> {
    let path = state
        .config_path
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "配置路径未初始化".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let config = current_saved_config(state);
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

fn persist_config_best_effort(state: &Arc<AppState>) {
    let _ = persist_config(state);
}

fn restart_calibration(state: &Arc<AppState>, clear_payload: bool) {
    let auto_sync = *state.auto_sync.lock().unwrap();
    {
        let mut sample_count = state.calibration_sample_count.lock().unwrap();
        *sample_count = 0;
    }
    {
        let mut started_at = state.calibration_started_at.lock().unwrap();
        *started_at = if auto_sync { Some(Instant::now()) } else { None };
    }
    {
        let mut stage = state.calibration_stage.lock().unwrap();
        *stage = if auto_sync {
            CalibrationStage::Calibrating
        } else {
            CalibrationStage::Idle
        };
    }
    if clear_payload {
        *state.last_payload.lock().unwrap() = None;
    }
}

fn stage_for_payload(state: &Arc<AppState>) -> CalibrationStage {
    *state.calibration_stage.lock().unwrap()
}

fn update_calibration_after_refresh(state: &Arc<AppState>, has_fresh_data: bool) {
    let auto_sync = *state.auto_sync.lock().unwrap();
    if !auto_sync {
        *state.calibration_stage.lock().unwrap() = CalibrationStage::Idle;
        *state.calibration_started_at.lock().unwrap() = None;
        *state.calibration_sample_count.lock().unwrap() = 0;
        return;
    }

    let mut stage = state.calibration_stage.lock().unwrap();
    if *stage == CalibrationStage::Idle {
        *stage = CalibrationStage::Calibrating;
        *state.calibration_started_at.lock().unwrap() = Some(Instant::now());
    }

    if *stage != CalibrationStage::Calibrating {
        return;
    }

    if has_fresh_data {
        let mut count = state.calibration_sample_count.lock().unwrap();
        *count += 1;
        if *count >= 3 {
            *stage = CalibrationStage::Stable;
            *state.calibration_started_at.lock().unwrap() = None;
            return;
        }
    }

    let timed_out = state
        .calibration_started_at
        .lock()
        .unwrap()
        .map(|started| started.elapsed() >= Duration::from_secs(CALIBRATION_TIMEOUT_SECS))
        .unwrap_or(false);

    if timed_out {
        *stage = if has_fresh_data {
            CalibrationStage::Stable
        } else {
            CalibrationStage::Degraded
        };
        *state.calibration_started_at.lock().unwrap() = None;
    }
}

fn sync_settings_payload(state: &Arc<AppState>) -> SyncSettingsPayload {
    SyncSettingsPayload {
        auto_sync: *state.auto_sync.lock().unwrap(),
        sync_interval_secs: *state.sync_interval_secs.lock().unwrap(),
        ntp_server: state.active_server.lock().unwrap().host.clone(),
        sync_mode: *state.sync_mode.lock().unwrap(),
        master_host: state.master_host.lock().unwrap().clone(),
        pair_code: state.pair_code.lock().unwrap().clone(),
        widget_enabled: *state.widget_enabled.lock().unwrap(),
        widget_scale: *state.widget_scale.lock().unwrap(),
        calibration_stage: *state.calibration_stage.lock().unwrap(),
        active_servers: available_servers(),
    }
}

fn next_sample_id(state: &Arc<AppState>) -> u64 {
    let mut cycle = state.cycle_count.lock().unwrap();
    *cycle += 1;
    *cycle
}

fn local_source_label(mode: SyncMode) -> String {
    match mode {
        SyncMode::LocalNtp => "本机 NTP".to_string(),
        SyncMode::Master => "局域网主机 / NTP".to_string(),
        SyncMode::Slave => "局域网从机".to_string(),
    }
}

fn build_local_ntp_payload(
    state: &Arc<AppState>,
    active_host: &str,
    previous_payload: Option<&NtpTimePayload>,
    sample_id: u64,
    sync_mode: SyncMode,
) -> Result<NtpTimePayload, String> {
    let mut server_latencies = HashMap::new();
    let mut active_samples: Vec<NtpSample> = Vec::new();

    for (host, _, _) in NTP_SERVERS {
        if *host != active_host {
            server_latencies.insert(
                host.to_string(),
                ServerLatency {
                    rtt: -1,
                    status: "standby".to_string(),
                },
            );
            continue;
        }

        match query_ntp(host) {
            Ok(sample) => {
                server_latencies.insert(
                    host.to_string(),
                    ServerLatency {
                        rtt: sample.rtt.round() as i64,
                        status: "ok".to_string(),
                    },
                );
                active_samples.push(sample);
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

    if active_samples.is_empty() && previous_payload.is_none() {
        return Err("当前 NTP 服务器无响应".to_string());
    }

    let has_fresh_data = !active_samples.is_empty();
    let ntp_offset = if has_fresh_data {
        let filtered = remove_outliers(&active_samples, 0.1);
        weighted_average(&filtered)
    } else {
        previous_payload.map(|p| p.ntp_offset).unwrap_or(0.0)
    };

    let ntp_rtt = if has_fresh_data {
        active_samples
            .iter()
            .map(|s| s.rtt)
            .fold(f64::MAX, |acc, value| acc.min(value))
            .round() as i64
    } else {
        -1
    };

    let corrected_time = (now_unix_ms() as f64 + ntp_offset) as u64;

    Ok(NtpTimePayload {
        server_time: corrected_time,
        ntp_offset,
        ntp_rtt,
        has_fresh_data,
        sample_id,
        ntp_server: active_host.to_string(),
        server_latencies,
        sync_mode,
        source_label: local_source_label(sync_mode),
        calibration_stage: stage_for_payload(state),
    })
}

fn normalize_master_host(host: &str) -> String {
    let trimmed = host.trim();
    if trimmed.is_empty() {
        return format!("127.0.0.1:{MASTER_PORT}");
    }
    if trimmed.contains(':') {
        trimmed.to_string()
    } else {
        format!("{trimmed}:{MASTER_PORT}")
    }
}

fn query_master_payload(
    state: &Arc<AppState>,
    master_host: &str,
    pair_code: &str,
    previous_payload: Option<&NtpTimePayload>,
    sample_id: u64,
) -> Result<NtpTimePayload, String> {
    let target = normalize_master_host(master_host);
    let start = Instant::now();
    let socket_addr = target
        .to_socket_addrs()
        .map_err(|e| format!("主机地址无效: {e}"))?
        .next()
        .ok_or_else(|| "无法解析主机地址".to_string())?;

    let mut stream =
        TcpStream::connect_timeout(&socket_addr, Duration::from_millis(1500)).map_err(|e| {
            if let Some(prev) = previous_payload {
                format!("LAN 主机无响应，沿用上次结果: {e} ({:.1}ms)", prev.ntp_offset)
            } else {
                format!("LAN 主机无响应: {e}")
            }
        })?;
    let _ = stream.set_read_timeout(Some(Duration::from_millis(1500)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(1500)));

    let request = MasterTimeRequest {
        pair_code: pair_code.to_string(),
        client_sent_ms: now_unix_ms(),
    };

    let request_json = serde_json::to_string(&request).map_err(|e| e.to_string())?;
    stream
        .write_all(request_json.as_bytes())
        .and_then(|_| stream.write_all(b"\n"))
        .map_err(|e| format!("发送同步请求失败: {e}"))?;
    stream.flush().map_err(|e| format!("发送同步请求失败: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("读取主机响应失败: {e}"))?;
    if line.trim().is_empty() {
        return Err("主机未返回有效同步数据".to_string());
    }

    let response: MasterTimeResponse =
        serde_json::from_str(&line).map_err(|e| format!("主机响应格式错误: {e}"))?;
    if !response.ok {
        return Err(response.message);
    }

    let upstream = response
        .payload
        .ok_or_else(|| "主机未返回时间数据".to_string())?;

    let rtt = start.elapsed().as_millis() as i64;
    let estimated_server_time = upstream.server_time as f64 + (rtt.max(0) as f64 / 2.0);
    let offset = estimated_server_time - now_unix_ms() as f64;

    let mut server_latencies = HashMap::new();
    server_latencies.insert(
        target.clone(),
        ServerLatency {
            rtt,
            status: "ok".to_string(),
        },
    );

    Ok(NtpTimePayload {
        server_time: estimated_server_time as u64,
        ntp_offset: offset,
        ntp_rtt: rtt,
        has_fresh_data: true,
        sample_id,
        ntp_server: target.clone(),
        server_latencies,
        sync_mode: SyncMode::Slave,
        source_label: format!("局域网主机 {target}"),
        calibration_stage: stage_for_payload(state),
    })
}

fn emit_payload(app_handle: Option<&AppHandle>, payload: &NtpTimePayload) {
    if let Some(app) = app_handle {
        let _ = app.emit("ntp-time", payload);
    }
}

fn refresh_payload_for_mode(
    app_handle: Option<&AppHandle>,
    state: &Arc<AppState>,
) -> Result<NtpTimePayload, String> {
    let sample_id = next_sample_id(state);
    let previous_payload = state.last_payload.lock().unwrap().clone();
    let sync_mode = *state.sync_mode.lock().unwrap();

    let result = match sync_mode {
        SyncMode::LocalNtp | SyncMode::Master => {
            let active_host = state.active_server.lock().unwrap().host.clone();
            build_local_ntp_payload(
                state,
                &active_host,
                previous_payload.as_ref(),
                sample_id,
                sync_mode,
            )
        }
        SyncMode::Slave => {
            let master_host = state.master_host.lock().unwrap().clone();
            let pair_code = state.pair_code.lock().unwrap().clone();
            query_master_payload(
                state,
                &master_host,
                &pair_code,
                previous_payload.as_ref(),
                sample_id,
            )
        }
    };

    match result {
        Ok(mut payload) => {
            update_calibration_after_refresh(state, payload.has_fresh_data);
            payload.calibration_stage = stage_for_payload(state);
            *state.last_payload.lock().unwrap() = Some(payload.clone());
            emit_payload(app_handle, &payload);
            if let Some(app) = app_handle {
                sync_startup_ui_state(app, state);
                sync_tray_state(app, state);
            }
            Ok(payload)
        }
        Err(err) => {
            if let Some(prev) = previous_payload {
                let mut stale = prev.clone();
                stale.sample_id = sample_id;
                stale.has_fresh_data = false;
                stale.ntp_rtt = -1;
                stale.sync_mode = sync_mode;
                stale.source_label = match sync_mode {
                    SyncMode::Slave => {
                        format!("局域网主机不可达，沿用上次结果 ({})", normalize_master_host(&state.master_host.lock().unwrap()))
                    }
                    _ => "网络无数据，沿用上次结果".to_string(),
                };
                stale.calibration_stage = stage_for_payload(state);
                *state.last_payload.lock().unwrap() = Some(stale.clone());
                emit_payload(app_handle, &stale);
                if let Some(app) = app_handle {
                    sync_startup_ui_state(app, state);
                    sync_tray_state(app, state);
                }
                Ok(stale)
            } else {
                update_calibration_after_refresh(state, false);
                if let Some(app) = app_handle {
                    sync_startup_ui_state(app, state);
                    sync_tray_state(app, state);
                }
                Err(err)
            }
        }
    }
}

fn app_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

fn load_saved_config(path: &Path) -> SavedConfig {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<SavedConfig>(&content).ok())
        .unwrap_or_default()
}

fn apply_saved_config(state: &Arc<AppState>, config: SavedConfig) {
    *state.auto_sync.lock().unwrap() = config.auto_sync;
    *state.sync_interval_secs.lock().unwrap() = config.sync_interval_secs.clamp(2, 3600);
    *state.active_server.lock().unwrap() = find_server(&config.ntp_server);
    *state.sync_mode.lock().unwrap() = config.sync_mode;
    *state.master_host.lock().unwrap() = normalize_master_host(&config.master_host);
    *state.pair_code.lock().unwrap() = if config.pair_code.trim().is_empty() {
        default_pair_code()
    } else {
        config.pair_code.trim().to_string()
    };
    *state.widget_enabled.lock().unwrap() = config.widget_enabled;
    *state.widget_position.lock().unwrap() = match (config.widget_x, config.widget_y) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    };
    *state.widget_scale.lock().unwrap() = clamp_widget_scale(config.widget_scale);
    restart_calibration(state, true);
}

fn should_collapse_to_tray(state: &Arc<AppState>) -> bool {
    *state.widget_enabled.lock().unwrap()
}

fn format_tray_tooltip(state: &Arc<AppState>) -> String {
    let version = env!("CARGO_PKG_VERSION");
    if let Some(payload) = state.last_payload.lock().unwrap().clone() {
        let sync_label = if payload.has_fresh_data {
            "已同步"
        } else {
            "沿用上次同步"
        };
        let latency = if payload.ntp_rtt >= 0 {
            format!("{}ms", payload.ntp_rtt)
        } else {
            "--".to_string()
        };
        return format!(
            "OpenTimeSync v{version}\n{sync_label} · {}\n偏移 {:+.1}ms · 延迟 {latency}",
            payload.source_label, payload.ntp_offset
        );
    }

    let stage_text = match *state.calibration_stage.lock().unwrap() {
        CalibrationStage::Idle => "未同步，当前显示本地时间",
        CalibrationStage::Calibrating => "正在建立首轮时间基线",
        CalibrationStage::Stable => "同步稳定中",
        CalibrationStage::Degraded => "网络一般，等待下一轮校准",
    };
    format!("OpenTimeSync v{version}\n{stage_text}")
}

fn sync_tray_state(app: &AppHandle, state: &Arc<AppState>) {
    let startup_completed = *state.startup_completed.lock().unwrap();
    let collapsed_to_tray = *state.collapsed_to_tray.lock().unwrap();
    let should_show = if !startup_completed {
        false
    } else {
        collapsed_to_tray
            || should_collapse_to_tray(state)
            || app
                .get_webview_window(MAIN_LABEL)
                .map(|window| !window.is_visible().unwrap_or(true))
                .unwrap_or(false)
    };

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_visible(should_show);
        let _ = tray.set_tooltip(Some(format_tray_tooltip(state)));
    }
}

fn sync_shell_surfaces(app: &AppHandle, state: &Arc<AppState>) {
    sync_widget_visibility(app, state);
    sync_tray_state(app, state);
}

fn hide_widget_window(app: &AppHandle, _state: &Arc<AppState>) {
    if let Some(widget) = app.get_webview_window(WIDGET_LABEL) {
        let _ = widget.hide();
    }
}

fn show_widget_window(app: &AppHandle, state: &Arc<AppState>) {
    if ensure_widget_window(app).is_err() {
        return;
    }
    if let Some(widget) = app.get_webview_window(WIDGET_LABEL) {
        let _ = position_widget_window(app, &widget, state);
        let _ = widget.show();
    }
}

fn should_finish_startup_ui(state: &Arc<AppState>) -> bool {
    if *state.startup_completed.lock().unwrap() {
        return true;
    }

    let elapsed_ms = state
        .calibration_started_at
        .lock()
        .unwrap()
        .map(|started| started.elapsed().as_millis() as u64)
        .unwrap_or(SPLASH_MAX_VISIBLE_MS);

    let auto_sync = *state.auto_sync.lock().unwrap();
    if !auto_sync {
        return elapsed_ms >= SPLASH_MIN_VISIBLE_MS;
    }

    if let Some(payload) = state.last_payload.lock().unwrap().clone() {
        if elapsed_ms >= SPLASH_MIN_VISIBLE_MS {
            if payload.calibration_stage == CalibrationStage::Stable {
                return true;
            }
            if payload.calibration_stage == CalibrationStage::Degraded {
                return true;
            }
            if payload.has_fresh_data {
                return true;
            }
        }
    }

    elapsed_ms >= SPLASH_MAX_VISIBLE_MS
}

fn complete_startup_ui(app: &AppHandle, state: &Arc<AppState>) {
    let mut completed = state.startup_completed.lock().unwrap();
    if *completed {
        return;
    }
    *completed = true;
    drop(completed);

    if let Some(splash) = app.get_webview_window(SPLASH_LABEL) {
        let _ = splash.close();
    }

    if let Some(main) = app.get_webview_window(MAIN_LABEL) {
        let _ = main.show();
        let _ = main.set_focus();
    }

    sync_shell_surfaces(app, state);
}

fn sync_startup_ui_state(app: &AppHandle, state: &Arc<AppState>) {
    if should_finish_startup_ui(state) {
        complete_startup_ui(app, state);
    }
}

fn schedule_startup_failsafe(app: AppHandle, state: Arc<AppState>) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(SPLASH_MAX_VISIBLE_MS + 600));
        complete_startup_ui(&app, &state);
    });
}

fn hide_main_window_to_tray(app: &AppHandle, state: &Arc<AppState>) {
    *state.collapsed_to_tray.lock().unwrap() = true;
    if let Some(main) = app.get_webview_window(MAIN_LABEL) {
        let _ = main.hide();
    }
    sync_shell_surfaces(app, state);
}

fn restore_main_window_internal(app: &AppHandle, state: &Arc<AppState>) -> Result<(), String> {
    complete_startup_ui(app, state);
    *state.collapsed_to_tray.lock().unwrap() = false;
    *state.widget_force_visible.lock().unwrap() = false;
    let main = app
        .get_webview_window(MAIN_LABEL)
        .ok_or_else(|| "主窗口不存在".to_string())?;
    let _ = main.show();
    let _ = main.unminimize();
    let _ = main.set_focus();
    hide_widget_window(app, state);
    sync_shell_surfaces(app, state);
    Ok(())
}

fn exit_application(app: &AppHandle) {
    if let Some(splash) = app.get_webview_window(SPLASH_LABEL) {
        let _ = splash.close();
    }
    if let Some(widget) = app.get_webview_window(WIDGET_LABEL) {
        let _ = widget.hide();
    }
    app.exit(0);
}

fn ensure_splash_window(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window(SPLASH_LABEL).is_some() {
        return Ok(());
    }

    let splash_builder = WebviewWindowBuilder::new(
        app,
        SPLASH_LABEL,
        WebviewUrl::App("splash.html".into()),
    )
    .title("OpenTimeSync Startup")
    .inner_size(760.0, 240.0)
    .resizable(false)
    .decorations(false)
    .background_color(Color(0, 0, 0, 0))
    .transparent(true);

    let splash = splash_builder
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .focused(false)
        .center()
        .build()
    .map_err(|e| e.to_string())?;

    let _ = splash.set_ignore_cursor_events(true);
    let _ = splash.set_background_color(Some(Color(0, 0, 0, 0)));

    Ok(())
}

fn ensure_tray_icon(app: &AppHandle, state: &Arc<AppState>) -> Result<(), String> {
    if app.tray_by_id(TRAY_ID).is_some() {
        sync_tray_state(app, state);
        return Ok(());
    }

    let show_item = MenuItem::with_id(app, TRAY_SHOW_ID, "显示主窗口", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let widget_item =
        MenuItem::with_id(app, TRAY_WIDGET_ID, "显示或隐藏悬浮挂件", true, None::<&str>)
            .map_err(|e| e.to_string())?;
    let quit_item = MenuItem::with_id(app, TRAY_EXIT_ID, "退出 OpenTimeSync", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let menu =
        Menu::with_items(app, &[&show_item, &widget_item, &quit_item]).map_err(|e| e.to_string())?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip(format_tray_tooltip(state))
        .show_menu_on_left_click(false);

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app).map_err(|e| e.to_string())?;
    sync_tray_state(app, state);
    Ok(())
}

fn widget_dimensions(state: &Arc<AppState>) -> (u32, u32) {
    let scale = *state.widget_scale.lock().unwrap() as f64 / 100.0;
    (
        (WIDGET_WIDTH * scale).round().max(156.0) as u32,
        (WIDGET_HEIGHT * scale).round().max(42.0) as u32,
    )
}

fn apply_widget_size(widget: &WebviewWindow, state: &Arc<AppState>) -> Result<(), String> {
    let (width, height) = widget_dimensions(state);
    widget
        .set_size(tauri::Size::Physical(tauri::PhysicalSize::new(width, height)))
        .map_err(|e| e.to_string())
}

fn position_widget_window(
    app: &AppHandle,
    widget: &WebviewWindow,
    state: &Arc<AppState>,
) -> Result<(), String> {
    let monitor = widget
        .current_monitor()
        .map_err(|e| e.to_string())?
        .or_else(|| app.primary_monitor().ok().flatten())
        .ok_or_else(|| "无法获取显示器".to_string())?;
    let work = monitor.work_area();
    let (width, height) = widget_dimensions(state);
    let work_right = work.position.x + work.size.width as i32;
    let work_bottom = work.position.y + work.size.height as i32;
    let min_x = work.position.x + 8;
    let min_y = work.position.y + 8;
    let max_x = (work_right - width as i32 - 8).max(min_x);
    let max_y = (work_bottom - height as i32 - 8).max(min_y);

    let stored = *state.widget_position.lock().unwrap();
    let (x, y) = if let Some((saved_x, saved_y)) = stored {
        (saved_x.clamp(min_x, max_x), saved_y.clamp(min_y, max_y))
    } else {
        let default_x = work_right - width as i32 - WIDGET_DEFAULT_RIGHT_MARGIN;
        let default_y = work_bottom - height as i32 - WIDGET_DEFAULT_BOTTOM_MARGIN;
        (default_x.clamp(min_x, max_x), default_y.clamp(min_y, max_y))
    };

    widget
        .set_position(PhysicalPosition::new(x.clamp(min_x, max_x), y.clamp(min_y, max_y)))
        .map_err(|e| e.to_string())
}

fn ensure_widget_window(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window(WIDGET_LABEL).is_some() {
        return Ok(());
    }

    let widget_builder =
        WebviewWindowBuilder::new(app, WIDGET_LABEL, WebviewUrl::App("index.html".into()))
        .title("OpenTimeSync Widget")
        .inner_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .resizable(false)
        .decorations(false)
        .background_color(Color(0, 0, 0, 0))
        .transparent(true);

    let widget = widget_builder
        .skip_taskbar(true)
        .always_on_top(true)
        .focused(false)
        .visible(false)
        .shadow(false)
        .build()
        .map_err(|e| e.to_string())?;

    let _ = widget.set_skip_taskbar(true);
    let _ = widget.set_background_color(Some(Color(0, 0, 0, 0)));
    apply_widget_size(&widget, &app.state::<Arc<AppState>>().inner().clone())?;
    Ok(())
}

fn sync_widget_visibility(app: &AppHandle, state: &Arc<AppState>) {
    if !*state.startup_completed.lock().unwrap() {
        if let Some(widget) = app.get_webview_window(WIDGET_LABEL) {
            let _ = widget.hide();
        }
        return;
    }

    let enabled = *state.widget_enabled.lock().unwrap();
    if app.get_webview_window(WIDGET_LABEL).is_none() && enabled && ensure_widget_window(app).is_err() {
        return;
    }

    if !enabled {
        *state.collapsed_to_tray.lock().unwrap() = false;
        *state.widget_dismissed.lock().unwrap() = false;
        *state.widget_force_visible.lock().unwrap() = false;
        hide_widget_window(app, state);
        return;
    }

    let collapsed_to_tray = *state.collapsed_to_tray.lock().unwrap();
    let dismissed = *state.widget_dismissed.lock().unwrap();
    let forced_visible = *state.widget_force_visible.lock().unwrap();
    let is_minimized = app
        .get_webview_window(MAIN_LABEL)
        .and_then(|window| window.is_minimized().ok())
        .unwrap_or(false);

    if (collapsed_to_tray || is_minimized || forced_visible) && !dismissed {
        show_widget_window(app, state);
    } else {
        hide_widget_window(app, state);
    }
}

fn schedule_widget_visibility_sync(app: AppHandle, state: Arc<AppState>) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(180));
        sync_shell_surfaces(&app, &state);
    });
}

fn toggle_widget_from_tray(app: &AppHandle, state: &Arc<AppState>) {
    if !*state.widget_enabled.lock().unwrap() {
        *state.widget_enabled.lock().unwrap() = true;
        *state.widget_dismissed.lock().unwrap() = false;
        *state.widget_force_visible.lock().unwrap() = true;
        persist_config_best_effort(state);
        sync_shell_surfaces(app, state);
        return;
    }

    let widget_visible = app
        .get_webview_window(WIDGET_LABEL)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);

    if widget_visible {
        *state.widget_force_visible.lock().unwrap() = false;
        *state.widget_dismissed.lock().unwrap() = true;
    } else {
        *state.widget_force_visible.lock().unwrap() = true;
        *state.widget_dismissed.lock().unwrap() = false;
    }
    sync_shell_surfaces(app, state);
}

fn attach_main_window_listener(app: &AppHandle, state: Arc<AppState>) {
    if let Some(main_window) = app.get_webview_window(MAIN_LABEL) {
        let app_handle = app.clone();
        main_window.on_window_event(move |event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                if should_collapse_to_tray(&state) {
                    api.prevent_close();
                    hide_main_window_to_tray(&app_handle, &state);
                }
            }
            WindowEvent::Focused(false) => {
                schedule_widget_visibility_sync(app_handle.clone(), state.clone());
            }
            WindowEvent::Focused(true) => {
                if let Some(widget) = app_handle.get_webview_window(WIDGET_LABEL) {
                    let _ = widget.hide();
                }
                schedule_widget_visibility_sync(app_handle.clone(), state.clone());
            }
            WindowEvent::Resized(_) | WindowEvent::Moved(_) => {
                schedule_widget_visibility_sync(app_handle.clone(), state.clone());
            }
            _ => {}
        });
    }
}

fn start_master_listener(state: Arc<AppState>) {
    std::thread::spawn(move || {
        let listener = match TcpListener::bind(("0.0.0.0", MASTER_PORT)) {
            Ok(listener) => listener,
            Err(err) => {
                eprintln!("master listener bind failed: {err}");
                return;
            }
        };
        let _ = listener.set_nonblocking(true);

        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(1500)));
                    let _ = stream.set_write_timeout(Some(Duration::from_millis(1500)));

                    let mut line = String::new();
                    let mut reader = BufReader::new(&mut stream);
                    let response = match reader.read_line(&mut line) {
                        Ok(_) if !line.trim().is_empty() => {
                            match serde_json::from_str::<MasterTimeRequest>(&line) {
                                Ok(request) => {
                                    let mode = *state.sync_mode.lock().unwrap();
                                    if mode != SyncMode::Master {
                                        MasterTimeResponse {
                                            ok: false,
                                            message: "当前设备未处于局域网主机模式".to_string(),
                                            payload: None,
                                        }
                                    } else {
                                        let expected_pair_code = state.pair_code.lock().unwrap().clone();
                                        if request.pair_code.trim() != expected_pair_code.trim() {
                                            MasterTimeResponse {
                                                ok: false,
                                                message: "配对码不正确".to_string(),
                                                payload: None,
                                            }
                                        } else {
                                            match state.last_payload.lock().unwrap().clone() {
                                                Some(payload) => MasterTimeResponse {
                                                    ok: true,
                                                    message: "ok".to_string(),
                                                    payload: Some(payload),
                                                },
                                                None => MasterTimeResponse {
                                                    ok: false,
                                                    message: "主机当前尚未完成同步".to_string(),
                                                    payload: None,
                                                },
                                            }
                                        }
                                    }
                                }
                                Err(err) => MasterTimeResponse {
                                    ok: false,
                                    message: format!("请求格式错误: {err}"),
                                    payload: None,
                                },
                            }
                        }
                        Ok(_) => MasterTimeResponse {
                            ok: false,
                            message: "空请求".to_string(),
                            payload: None,
                        },
                        Err(err) => MasterTimeResponse {
                            ok: false,
                            message: format!("读取请求失败: {err}"),
                            payload: None,
                        },
                    };

                    if let Ok(json) = serde_json::to_string(&response) {
                        let _ = stream.write_all(json.as_bytes());
                        let _ = stream.write_all(b"\n");
                        let _ = stream.flush();
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(200));
                }
                Err(err) => {
                    eprintln!("master listener error: {err}");
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }
    });
}

fn run_ntp_loop(app_handle: AppHandle, app_state: Arc<AppState>) {
    std::thread::spawn(move || {
        let mut last_auto_sync_at: Option<Instant> = None;

        loop {
            sync_startup_ui_state(&app_handle, &app_state);
            sync_shell_surfaces(&app_handle, &app_state);

            let auto_sync = *app_state.auto_sync.lock().unwrap();
            let stage = *app_state.calibration_stage.lock().unwrap();
            let configured_interval = *app_state.sync_interval_secs.lock().unwrap();
            let effective_interval = if stage.is_calibrating() {
                CALIBRATION_INTERVAL_SECS
            } else {
                configured_interval
            };

            let should_sync = auto_sync
                && match last_auto_sync_at {
                    None => true,
                    Some(last) => last.elapsed() >= Duration::from_secs(effective_interval),
                };

            if should_sync {
                let _ = refresh_payload_for_mode(Some(&app_handle), &app_state);
                last_auto_sync_at = Some(Instant::now());
            }

            std::thread::sleep(Duration::from_millis(500));
        }
    });
}

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
            privileges: [LUID_AND_ATTRIBUTES {
                luid,
                attributes: SE_PRIVILEGE_ENABLED,
            }],
        };
        let mut prev: TOKEN_PRIVILEGES = std::mem::zeroed();
        let mut ret: u32 = 0;
        AdjustTokenPrivileges(
            token,
            0,
            &tp,
            std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
            &mut prev,
            &mut ret,
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
        let diy = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
        if rem < diy {
            break;
        }
        rem -= diy;
        y += 1;
    }
    let md: [i64; 12] = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut mon: u16 = 1;
    for &d in &md {
        if rem < d {
            break;
        }
        rem -= d;
        mon += 1;
    }

    enable_privilege().map_err(|e| format!("权限不足: {e}"))?;

    let st = SYSTEMTIME {
        wYear: y as u16,
        wMonth: mon,
        wDayOfWeek: 0,
        wDay: (rem + 1) as u16,
        wHour: hours,
        wMinute: minutes,
        wSecond: seconds,
        wMilliseconds: millis,
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
fn get_runtime_context(window: Window) -> RuntimeContextPayload {
    RuntimeContextPayload {
        is_widget: window.label() == WIDGET_LABEL,
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
fn get_sync_settings(state: State<'_, Arc<AppState>>) -> SyncSettingsPayload {
    sync_settings_payload(state.inner())
}

#[tauri::command]
fn sync_system_time(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let payload = state.last_payload.lock().unwrap().clone();
    match payload {
        Some(payload) => {
            set_windows_system_time(payload.server_time)?;
            Ok("系统时间已按当前校准值同步".to_string())
        }
        None => Err("尚无可用同步结果".to_string()),
    }
}

#[tauri::command]
fn sync_ntp_now(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let payload = refresh_payload_for_mode(Some(&app), state.inner())?;
    if payload.has_fresh_data {
        Ok(format!(
            "同步完成 来源 {} 偏移 {:.1}ms 延迟 {}ms",
            payload.source_label, payload.ntp_offset, payload.ntp_rtt
        ))
    } else {
        Err("当前网络无新数据，已沿用上次有效同步结果".to_string())
    }
}

#[tauri::command]
fn set_auto_sync(state: State<'_, Arc<AppState>>, enabled: bool) {
    *state.auto_sync.lock().unwrap() = enabled;
    restart_calibration(state.inner(), false);
    persist_config_best_effort(state.inner());
}

#[tauri::command]
fn get_auto_sync(state: State<'_, Arc<AppState>>) -> bool {
    *state.auto_sync.lock().unwrap()
}

#[tauri::command]
fn set_sync_interval(state: State<'_, Arc<AppState>>, seconds: u64) {
    *state.sync_interval_secs.lock().unwrap() = seconds.clamp(2, 3600);
    persist_config_best_effort(state.inner());
}

#[tauri::command]
fn get_sync_interval(state: State<'_, Arc<AppState>>) -> u64 {
    *state.sync_interval_secs.lock().unwrap()
}

#[tauri::command]
fn set_ntp_server(state: State<'_, Arc<AppState>>, server: String) -> bool {
    for item in available_servers() {
        if item.host == server {
            *state.active_server.lock().unwrap() = item;
            restart_calibration(state.inner(), true);
            persist_config_best_effort(state.inner());
            return true;
        }
    }
    false
}

#[tauri::command]
fn set_sync_mode(state: State<'_, Arc<AppState>>, mode: SyncMode) {
    *state.sync_mode.lock().unwrap() = mode;
    restart_calibration(state.inner(), true);
    persist_config_best_effort(state.inner());
}

#[tauri::command]
fn set_master_host(state: State<'_, Arc<AppState>>, host: String) {
    *state.master_host.lock().unwrap() = normalize_master_host(&host);
    restart_calibration(state.inner(), true);
    persist_config_best_effort(state.inner());
}

#[tauri::command]
fn set_pair_code(state: State<'_, Arc<AppState>>, code: String) {
    let trimmed = code.trim();
    *state.pair_code.lock().unwrap() = if trimmed.is_empty() {
        default_pair_code()
    } else {
        trimmed.to_string()
    };
    persist_config_best_effort(state.inner());
}

#[tauri::command]
fn set_widget_enabled(app: AppHandle, state: State<'_, Arc<AppState>>, enabled: bool) {
    *state.widget_enabled.lock().unwrap() = enabled;
    if !enabled {
        *state.widget_dismissed.lock().unwrap() = true;
        *state.widget_force_visible.lock().unwrap() = false;
    } else {
        *state.widget_dismissed.lock().unwrap() = false;
    }
    persist_config_best_effort(state.inner());
    sync_shell_surfaces(&app, state.inner());
}

#[tauri::command]
fn set_widget_scale(app: AppHandle, state: State<'_, Arc<AppState>>, scale: u32) -> Result<(), String> {
    *state.widget_scale.lock().unwrap() = clamp_widget_scale(scale);
    persist_config_best_effort(state.inner());
    if let Some(widget) = app.get_webview_window(WIDGET_LABEL) {
        apply_widget_size(&widget, state.inner())?;
        position_widget_window(&app, &widget, state.inner())?;
    }
    Ok(())
}

#[tauri::command]
fn update_widget_position(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    x: i32,
    y: i32,
) -> Result<(), String> {
    let widget = app
        .get_webview_window(WIDGET_LABEL)
        .ok_or_else(|| "挂件窗口不存在".to_string())?;
    *state.widget_position.lock().unwrap() = Some((x, y));
    persist_config_best_effort(state.inner());
    position_widget_window(&app, &widget, state.inner())
}

#[tauri::command]
fn dismiss_widget(app: AppHandle, state: State<'_, Arc<AppState>>) {
    *state.widget_dismissed.lock().unwrap() = true;
    *state.widget_enabled.lock().unwrap() = false;
    *state.collapsed_to_tray.lock().unwrap() = false;
    *state.widget_force_visible.lock().unwrap() = false;
    persist_config_best_effort(state.inner());
    sync_tray_state(&app, state.inner());
    hide_widget_window(&app, state.inner());
}

#[tauri::command]
fn save_widget_position(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let widget = app
        .get_webview_window(WIDGET_LABEL)
        .ok_or_else(|| "挂件窗口不存在".to_string())?;
    let position = widget.outer_position().map_err(|e| e.to_string())?;
    *state.widget_position.lock().unwrap() = Some((position.x, position.y));
    persist_config_best_effort(state.inner());
    Ok(())
}

#[tauri::command]
fn start_widget_drag(app: AppHandle) -> Result<(), String> {
    let widget = app
        .get_webview_window(WIDGET_LABEL)
        .ok_or_else(|| "挂件窗口不存在".to_string())?;
    widget.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_ntp_status(state: State<'_, Arc<AppState>>) -> Option<NtpTimePayload> {
    state.last_payload.lock().unwrap().clone()
}

#[tauri::command]
fn minimize_window(window: Window, state: State<'_, Arc<AppState>>) {
    if should_collapse_to_tray(state.inner()) {
        hide_main_window_to_tray(&window.app_handle(), state.inner());
        return;
    }
    let _ = window.minimize();
    schedule_widget_visibility_sync(window.app_handle().clone(), state.inner().clone());
}

#[tauri::command]
fn maximize_window(window: Window, state: State<'_, Arc<AppState>>) {
    if let Ok(true) = window.is_maximized() {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
    let app = window.app_handle().clone();
    if let Some(widget) = app.get_webview_window(WIDGET_LABEL) {
        let _ = widget.hide();
    }
    schedule_widget_visibility_sync(app, state.inner().clone());
}

#[tauri::command]
fn close_window(window: Window, state: State<'_, Arc<AppState>>) {
    if should_collapse_to_tray(state.inner()) {
        hide_main_window_to_tray(&window.app_handle(), state.inner());
    } else {
        let _ = window.close();
    }
}

#[tauri::command]
fn start_drag(window: Window) {
    let _ = window.start_dragging();
}

#[tauri::command]
fn restore_main_window(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    restore_main_window_internal(&app, state.inner())
}

#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn set_update_status(
    state: &Arc<AppState>,
    phase: impl Into<String>,
    current_version: impl Into<String>,
    version: Option<String>,
    message: impl Into<String>,
    notes: Option<String>,
    downloaded_bytes: Option<u64>,
    total_bytes: Option<u64>,
) -> UpdateStatusPayload {
    let status = UpdateStatusPayload {
        phase: phase.into(),
        current_version: current_version.into(),
        version,
        message: message.into(),
        notes,
        downloaded_bytes,
        total_bytes,
    };
    *state.update_status.lock().unwrap() = status.clone();
    status
}

fn format_update_error(err: impl std::fmt::Display) -> String {
    let msg = err.to_string();
    if msg.contains("signature") {
        "更新签名无效，请先检查发布签名配置".to_string()
    } else if msg.contains("ReleaseNotFound") {
        "未找到可用更新".to_string()
    } else {
        msg
    }
}

#[tauri::command]
fn get_update_status(state: State<'_, Arc<AppState>>) -> UpdateStatusPayload {
    state.update_status.lock().unwrap().clone()
}

#[tauri::command]
async fn check_for_update(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<UpdateStatusPayload, String> {
    let app_state = state.inner().clone();
    let current_version = app.package_info().version.to_string();

    set_update_status(
        &app_state,
        "checking",
        current_version.clone(),
        None,
        "正在检查更新...",
        None,
        None,
        None,
    );
    *app_state.downloaded_update.lock().unwrap() = None;

    let updater = app
        .updater_builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(format_update_error)?;

    match updater.check().await.map_err(format_update_error)? {
        Some(update) => Ok(set_update_status(
            &app_state,
            "available",
            current_version,
            Some(update.version.clone()),
            format!("发现新版本 v{}，点击下载并安装", update.version),
            update.body.clone(),
            None,
            None,
        )),
        None => Ok(set_update_status(
            &app_state,
            "upToDate",
            current_version,
            None,
            "已是最新版本",
            None,
            None,
            None,
        )),
    }
}

#[tauri::command]
fn download_available_update(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let app_state = state.inner().clone();
    let current_version = app.package_info().version.to_string();

    {
        let status = app_state.update_status.lock().unwrap().clone();
        if matches!(status.phase.as_str(), "checking" | "downloading" | "installing") {
            return Err("更新任务正在进行中".to_string());
        }
    }

    *app_state.downloaded_update.lock().unwrap() = None;

    set_update_status(
        &app_state,
        "downloading",
        current_version.clone(),
        None,
        "正在准备下载安装...",
        app_state.update_status.lock().unwrap().notes.clone(),
        Some(0),
        None,
    );

    tauri::async_runtime::spawn(async move {
        let result: Result<(), String> = async {
            let updater = app
                .updater_builder()
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(format_update_error)?;

            let update = updater
                .check()
                .await
                .map_err(format_update_error)?
                .ok_or_else(|| "已是最新版本".to_string())?;

            let version = update.version.clone();
            let notes = update.body.clone();
            let mut downloaded_bytes = 0u64;

            set_update_status(
                &app_state,
                "downloading",
                current_version.clone(),
                Some(version.clone()),
                format!("正在下载 v{version}"),
                notes.clone(),
                Some(0),
                None,
            );

            let bytes = update
                .download(
                    |chunk_length, content_length| {
                        downloaded_bytes += chunk_length as u64;
                        let message = match content_length {
                            Some(total) if total > 0 => {
                                let progress =
                                    (downloaded_bytes as f64 / total as f64 * 100.0).round();
                                format!("正在下载 v{version} ({progress}%)")
                            }
                            _ => format!("正在下载 v{version}"),
                        };
                        set_update_status(
                            &app_state,
                            "downloading",
                            current_version.clone(),
                            Some(version.clone()),
                            message,
                            notes.clone(),
                            Some(downloaded_bytes),
                            content_length,
                        );
                    },
                    || {},
                )
                .await
                .map_err(format_update_error)?;

            *app_state.downloaded_update.lock().unwrap() = Some(DownloadedUpdate {
                update,
                bytes,
                notes: notes.clone(),
            });

            set_update_status(
                &app_state,
                "downloaded",
                current_version.clone(),
                Some(version.clone()),
                format!("v{version} 下载完成，点击安装并重启"),
                notes,
                Some(downloaded_bytes),
                Some(downloaded_bytes),
            );

            Ok(())
        }
        .await;

        if let Err(err) = result {
            set_update_status(
                &app_state,
                "error",
                current_version,
                None,
                format!("更新失败：{err}"),
                None,
                None,
                None,
            );
        }
    });

    Ok(())
}

#[tauri::command]
fn install_downloaded_update(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let app_state = state.inner().clone();
    let current_version = app_state.update_status.lock().unwrap().current_version.clone();
    let downloaded = app_state.downloaded_update.lock().unwrap().take();

    let downloaded = downloaded.ok_or_else(|| "尚未下载可安装的更新".to_string())?;
    let version = downloaded.update.version.clone();

    set_update_status(
        &app_state,
        "installing",
        current_version.clone(),
        Some(version.clone()),
        format!("正在启动安装器，应用将自动关闭并在安装完成后回到 v{version}"),
        downloaded.notes.clone(),
        None,
        None,
    );

    tauri::async_runtime::spawn(async move {
        std::thread::sleep(Duration::from_millis(250));
        if let Err(err) = downloaded.update.install(&downloaded.bytes) {
            set_update_status(
                &app_state,
                "error",
                current_version,
                Some(version),
                format!("启动安装器失败：{}", format_update_error(err)),
                downloaded.notes,
                None,
                None,
            );
        }
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = SavedConfig::default();
    let app_state = Arc::new(AppState {
        active_server: Mutex::new(find_server(&config.ntp_server)),
        last_payload: Mutex::new(None),
        cycle_count: Mutex::new(0),
        auto_sync: Mutex::new(config.auto_sync),
        sync_interval_secs: Mutex::new(config.sync_interval_secs),
        sync_mode: Mutex::new(config.sync_mode),
        master_host: Mutex::new(config.master_host),
        pair_code: Mutex::new(config.pair_code),
        widget_enabled: Mutex::new(config.widget_enabled),
        collapsed_to_tray: Mutex::new(false),
        widget_dismissed: Mutex::new(false),
        widget_force_visible: Mutex::new(false),
        widget_position: Mutex::new(match (config.widget_x, config.widget_y) {
            (Some(x), Some(y)) => Some((x, y)),
            _ => None,
        }),
        widget_scale: Mutex::new(clamp_widget_scale(config.widget_scale)),
        calibration_stage: Mutex::new(CalibrationStage::Calibrating),
        calibration_started_at: Mutex::new(Some(Instant::now())),
        calibration_sample_count: Mutex::new(0),
        startup_completed: Mutex::new(false),
        config_path: Mutex::new(None),
        update_status: Mutex::new(UpdateStatusPayload {
            phase: "idle".to_string(),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            version: None,
            message: String::new(),
            notes: None,
            downloaded_bytes: None,
            total_bytes: None,
        }),
        downloaded_update: Mutex::new(None),
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .on_menu_event(|app, event| {
            if event.id() == TRAY_SHOW_ID {
                let state = app.state::<Arc<AppState>>().inner().clone();
                let _ = restore_main_window_internal(app, &state);
            } else if event.id() == TRAY_WIDGET_ID {
                let state = app.state::<Arc<AppState>>().inner().clone();
                toggle_widget_from_tray(app, &state);
            } else if event.id() == TRAY_EXIT_ID {
                exit_application(app);
            }
        })
        .on_tray_icon_event(|app, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            }
            | TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => {
                let state = app.state::<Arc<AppState>>().inner().clone();
                let _ = restore_main_window_internal(app, &state);
            }
            _ => {}
        })
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            ping,
            get_runtime_context,
            get_sync_settings,
            get_version,
            sync_ntp_now,
            sync_system_time,
            set_auto_sync,
            get_auto_sync,
            set_sync_interval,
            get_sync_interval,
            set_ntp_server,
            set_sync_mode,
            set_master_host,
            set_pair_code,
            set_widget_enabled,
            set_widget_scale,
            update_widget_position,
            dismiss_widget,
            save_widget_position,
            start_widget_drag,
            get_ntp_status,
            minimize_window,
            maximize_window,
            close_window,
            start_drag,
            restore_main_window,
            get_update_status,
            check_for_update,
            download_available_update,
            install_downloaded_update,
        ])
        .setup(move |app| {
            let path = app_config_path(&app.handle())?;
            *app_state.config_path.lock().unwrap() = Some(path.clone());
            let loaded = load_saved_config(&path);
            apply_saved_config(&app_state, loaded);

            if let Some(main) = app.get_webview_window(MAIN_LABEL) {
                let _ = main.hide();
            }
            ensure_splash_window(&app.handle())?;
            ensure_tray_icon(&app.handle(), &app_state)?;
            ensure_widget_window(&app.handle())?;
            sync_shell_surfaces(&app.handle(), &app_state);
            sync_startup_ui_state(&app.handle(), &app_state);
            schedule_startup_failsafe(app.handle().clone(), app_state.clone());
            attach_main_window_listener(&app.handle(), app_state.clone());
            start_master_listener(app_state.clone());
            run_ntp_loop(app.handle().clone(), app_state.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
