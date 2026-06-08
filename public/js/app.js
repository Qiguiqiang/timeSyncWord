function dbg(msg) {
  console.log('[TS]', msg);
}

const DOM = {
  body: document.body,
  titlebar: document.querySelector('.titlebar'),
  container: document.querySelector('.container'),
  hours: document.getElementById('timeHours'),
  mins: document.getElementById('timeMins'),
  secs: document.getElementById('timeSecs'),
  ms: document.getElementById('currentMs'),
  utcDisplay: document.getElementById('utcDisplay'),
  offsetDisplay: document.getElementById('offsetDisplay'),
  offsetValue: document.getElementById('offsetValue'),
  precisionTier: document.getElementById('precisionTier'),
  precisionError: document.getElementById('precisionError'),
  sampleCount: document.getElementById('sampleCount'),
  statusDot: document.getElementById('statusDot'),
  statusLabel: document.getElementById('statusLabel'),
  tzDisplay: document.getElementById('tzDisplay'),
  tzPanel: document.getElementById('tzPanel'),
  ntpName: document.getElementById('ntpName'),
  ntpRttLabel: document.getElementById('ntpRttLabel'),
  ntpPanel: document.getElementById('ntpPanel'),
  ntpSelector: document.getElementById('ntpSelector'),
  btnSettings: document.getElementById('btnSettings'),
  settingsPanel: document.getElementById('settingsPanel'),
  btnSyncNtpNow: document.getElementById('btnSyncNtpNow'),
  btnSyncNow: document.getElementById('btnSyncNow'),
  chkAutoSync: document.getElementById('chkAutoSync'),
  syncInterval: document.getElementById('syncInterval'),
  widgetScale: document.getElementById('widgetScale'),
  widgetScaleValue: document.getElementById('widgetScaleValue'),
  syncMode: document.getElementById('syncMode'),
  masterHost: document.getElementById('masterHost'),
  masterHostRow: document.getElementById('masterHostRow'),
  pairCode: document.getElementById('pairCode'),
  modeHint: document.getElementById('modeHint'),
  chkWidgetEnabled: document.getElementById('chkWidgetEnabled'),
  syncStatus: document.getElementById('syncStatus'),
  versionNum: document.getElementById('versionNum'),
  btnCheckUpdate: document.getElementById('btnCheckUpdate'),
  updateStatus: document.getElementById('updateStatus'),
  updateModal: document.getElementById('updateModal'),
  updateCurrentVersion: document.getElementById('updateCurrentVersion'),
  updateLatestVersion: document.getElementById('updateLatestVersion'),
  updateNotes: document.getElementById('updateNotes'),
  updateProgressBar: document.getElementById('updateProgressBar'),
  updateProgressText: document.getElementById('updateProgressText'),
  btnUpdateLater: document.getElementById('btnUpdateLater'),
  btnUpdateNow: document.getElementById('btnUpdateNow'),
  bootOverlay: document.getElementById('bootOverlay'),
  bootStatusText: document.getElementById('bootStatusText'),
  bootProgressBar: document.getElementById('bootProgressBar'),
  bootCaption: document.getElementById('bootCaption'),
  widgetShell: document.getElementById('widgetShell'),
  widgetClose: document.getElementById('widgetClose'),
  widgetBadge: document.getElementById('widgetBadge'),
  widgetTime: document.getElementById('widgetTime')
};

const TIMEZONES = [
  { value: 'UTC', label: 'UTC+0', name: 'UTC' },
  { value: 'Asia/Shanghai', label: 'UTC+8', name: 'Shanghai' },
  { value: 'Asia/Tokyo', label: 'UTC+9', name: 'Tokyo' },
  { value: 'Asia/Singapore', label: 'UTC+8', name: 'Singapore' },
  { value: 'Asia/Dubai', label: 'UTC+4', name: 'Dubai' },
  { value: 'Asia/Kolkata', label: 'UTC+5:30', name: 'Kolkata' },
  { value: 'Europe/Moscow', label: 'UTC+3', name: 'Moscow' },
  { value: 'Europe/Berlin', label: 'UTC+1', name: 'Berlin' },
  { value: 'Europe/London', label: 'UTC+0', name: 'London' },
  { value: 'America/New_York', label: 'UTC-5', name: 'New York' },
  { value: 'America/Chicago', label: 'UTC-6', name: 'Chicago' },
  { value: 'America/Los_Angeles', label: 'UTC-8', name: 'Los Angeles' },
  { value: 'Pacific/Auckland', label: 'UTC+13', name: 'Auckland' },
  { value: 'Australia/Sydney', label: 'UTC+11', name: 'Sydney' }
];

const NTP_SERVERS = [
  { host: 'ntp.tencent.com', name: 'Tencent', label: '腾讯云' },
  { host: 'ntp.aliyun.com', name: 'Aliyun', label: '阿里云' },
  { host: 'time.asia.apple.com', name: 'Apple', label: 'Apple Asia' },
  { host: 'time.google.com', name: 'Google', label: 'Google' },
  { host: 'pool.ntp.org', name: 'Pool', label: 'pool.ntp.org' }
];

const MODE_HINTS = {
  localNtp: '本机直接通过NTP对时，不经过局域网主从转发。',
  master: '本机先通过NTP校准，再把时间结果提供给局域网从机。',
  slave: '当前设备从局域网主机获取同步时间，主机异常时会沿用上次有效结果。'
};

const WIDGET_BASE_WIDTH = 172;

const State = {
  isWidget: false,
  appVersion: '',
  timezone: '',
  offset: 0,
  ntpOffset: 0,
  offsetStd: 0,
  ntpRtt: -1,
  samples: [],
  maxSamples: 20,
  hasNtpData: false,
  hasFreshData: false,
  isSynced: false,
  lastSampleId: -1,
  syncBase: Date.now(),
  perfBase: performance.now(),
  autoSync: true,
  syncIntervalSecs: 5,
  widgetScale: 100,
  syncMode: 'localNtp',
  masterHost: '127.0.0.1:36363',
  pairCode: '',
  widgetEnabled: false,
  calibrationStage: 'calibrating',
  sourceLabel: '本地时间',
  ntpServer: 'ntp.tencent.com',
  serverLatencies: {},
  activeServers: [],
  currentUpdatePhase: 'idle',
  bootHidden: false,
  bootStartedAt: Date.now()
};

let ntpPollTimer = 0;
let updatePollTimer = 0;
let lastSec = -1;
let lastMin = -1;
let lastHour = -1;
let bootTimer = 0;
let widgetDrag = null;
let widgetClickGuardUntil = 0;
let widgetScalePersistTimer = 0;
let widgetSettingsPollTimer = 0;

function cls(value, warn, danger) {
  return value < warn ? 'good' : value < danger ? 'warning' : 'danger';
}

function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

function calculateStdDev(values) {
  if (values.length < 2) return 0;
  const mean = values.reduce((a, b) => a + b, 0) / values.length;
  const variance = values.reduce((sum, value) => sum + Math.pow(value - mean, 2), 0) / values.length;
  return Math.sqrt(variance);
}

function getPrecisionTier() {
  const value = State.offsetStd;
  if (value < 2) return 'S+';
  if (value < 5) return 'S';
  if (value < 10) return 'S-';
  if (value < 30) return 'A';
  if (value < 50) return 'B';
  if (value < 100) return 'C';
  return 'D';
}

function getPrecisionClass(tier) {
  if (tier === 'S+' || tier === 'S') return 'good';
  if (tier === 'S-' || tier === 'A') return 'warning';
  return 'danger';
}

function resetSyncState() {
  State.offset = 0;
  State.ntpOffset = 0;
  State.offsetStd = 0;
  State.ntpRtt = -1;
  State.samples = [];
  State.hasNtpData = false;
  State.hasFreshData = false;
  State.isSynced = false;
  State.lastSampleId = -1;
  State.syncBase = Date.now();
  State.perfBase = performance.now();
  State.calibrationStage = State.autoSync ? 'calibrating' : 'idle';
  State.sourceLabel = State.syncMode === 'slave' ? '等待主机同步' : '本地时间';
}

function hideBootOverlay() {
  if (State.bootHidden || !DOM.bootOverlay) return;
  State.bootHidden = true;
  DOM.body.classList.remove('boot-active');
  DOM.bootOverlay.classList.add('hidden');
  if (bootTimer) {
    clearTimeout(bootTimer);
    bootTimer = 0;
  }
}

function shouldHideBootOverlay() {
  if (State.isWidget) return true;
  if (!State.autoSync) return true;
  if (State.calibrationStage === 'stable' && State.samples.length >= 3) return true;
  if (State.calibrationStage === 'degraded' && State.hasNtpData) return true;
  if (Date.now() - State.bootStartedAt >= 35000) return true;
  return false;
}

function maybeFinishBootOverlay(delay = 0) {
  if (!shouldHideBootOverlay()) return;
  if (delay > 0) {
    setTimeout(() => {
      if (shouldHideBootOverlay()) hideBootOverlay();
    }, delay);
    return;
  }
  hideBootOverlay();
}

function bootProgressForState() {
  if (State.hasNtpData && State.isSynced && State.samples.length >= 3) return 100;
  if (State.hasNtpData && State.isSynced) return 82;
  if (State.hasNtpData) return 68;
  if (State.autoSync && State.calibrationStage === 'calibrating') return 42;
  return 18;
}

function renderBootState() {
  if (!DOM.bootOverlay || State.bootHidden) return;

  let status = '正在初始化同步引擎...';
  let caption = 'OPEN TIME SYNC 正在建立首轮高精度时间基线';

  if (!State.autoSync) {
    status = '正在载入本地时间界面...';
    caption = '当前未开启自动同步，启动后显示本地系统时间';
  } else if (State.syncMode === 'slave' && !State.hasNtpData) {
    status = '正在等待局域网主机响应...';
    caption = '通过局域网主机获取首轮校准结果';
  } else if (State.hasNtpData && State.isSynced && State.samples.length >= 3) {
    status = '首轮校准完成，正在进入主界面...';
    caption = `${State.sourceLabel} 已建立稳定时间基线 · 精度层级已可用`;
  } else if (State.hasNtpData && State.isSynced) {
    status = '已拿到时间样本，继续校准中...';
    caption = `继续采样以稳定偏移和精度 · 当前 ${State.samples.length}/3 · ${State.sourceLabel}`;
  } else if (State.calibrationStage === 'degraded') {
    status = '网络质量一般，使用当前可用结果启动...';
    caption = '未达到理想校准质量，但不会阻塞进入主界面';
  }

  DOM.bootStatusText.textContent = status;
  DOM.bootCaption.textContent = caption;
  DOM.bootProgressBar.style.width = `${bootProgressForState()}%`;
}

function getSyncTime() {
  return State.isSynced ? State.syncBase + (performance.now() - State.perfBase) : Date.now();
}

function calculateOffset() {
  if (State.samples.length === 0) {
    State.isSynced = false;
    return;
  }

  if (State.samples.length < 3) {
    State.offset = State.samples[State.samples.length - 1];
    State.offsetStd = 0;
    State.syncBase = Date.now() + State.offset;
    State.perfBase = performance.now();
    State.isSynced = true;
    return;
  }

  const sorted = [...State.samples].sort((a, b) => a - b);
  const cutoff = Math.floor(sorted.length * 0.1);
  const filtered = sorted.slice(cutoff, sorted.length - cutoff);
  const base = filtered.length ? filtered : sorted;

  State.offset = base.reduce((a, b) => a + b, 0) / base.length;
  State.offsetStd = calculateStdDev(base);
  State.syncBase = Date.now() + State.offset;
  State.perfBase = performance.now();
  State.isSynced = true;
}

function handleTime(payload) {
  if (!payload || typeof payload.ntpOffset === 'undefined') return;

  const sampleId = typeof payload.sampleId === 'number' ? payload.sampleId : State.lastSampleId;
  const isNewSample = sampleId !== State.lastSampleId;
  const hasFreshData = payload.hasFreshData !== false;

  State.hasNtpData = true;
  State.hasFreshData = hasFreshData;
  State.ntpOffset = Number(payload.ntpOffset) || 0;
  State.ntpRtt = typeof payload.ntpRtt === 'number' ? payload.ntpRtt : -1;
  State.ntpServer = payload.ntpServer || State.ntpServer;
  State.serverLatencies = payload.serverLatencies || {};
  State.sourceLabel = payload.sourceLabel || State.sourceLabel;
  State.syncMode = payload.syncMode || State.syncMode;
  State.calibrationStage = payload.calibrationStage || State.calibrationStage;

  if (isNewSample && hasFreshData) {
    State.samples.push(State.ntpOffset);
    if (State.samples.length > State.maxSamples) State.samples.shift();
    calculateOffset();
  }

  State.lastSampleId = sampleId;
  renderModeUI();
  updateUI();
  renderBootState();
  maybeFinishBootOverlay(380);
}

function setStatus(stateClass, label) {
  DOM.statusDot.className = 'status-dot ' + stateClass;
  DOM.statusLabel.textContent = label;
}

function getStageStatusText() {
  if (!State.hasNtpData) {
    if (!State.autoSync) {
      return { dot: 'connecting', text: '未同步，当前显示本地时间' };
    }
    if (State.syncMode === 'slave') {
      return { dot: 'connecting', text: '等待局域网主机同步，当前显示本地时间' };
    }
    return { dot: 'connecting', text: '校准中，当前显示本地时间' };
  }

  if (State.hasFreshData && State.isSynced) {
    if (State.calibrationStage === 'calibrating') {
      return {
        dot: 'connecting',
        text: `${State.sourceLabel} 校准中 采样 ${State.samples.length}/3`
      };
    }
    if (State.calibrationStage === 'degraded') {
      return {
        dot: 'connecting',
        text: `${State.sourceLabel} 已同步，但质量降级 偏移 ${State.ntpOffset.toFixed(1)}ms`
      };
    }
    return {
      dot: 'synced',
      text: `${State.sourceLabel} 已同步 偏移 ${State.ntpOffset.toFixed(1)}ms 精度 ±${State.offsetStd.toFixed(1)}ms`
    };
  }

  if (State.isSynced) {
    return {
      dot: 'connecting',
      text: `${State.sourceLabel} 暂无新数据，沿用上次偏移 ${State.ntpOffset.toFixed(1)}ms`
    };
  }

  return { dot: 'connecting', text: '未同步，当前显示本地时间' };
}

function getNtpRttClass(rtt) {
  if (rtt <= 0) return '';
  if (rtt < 30) return 'ok';
  if (rtt < 100) return 'warning';
  return 'timeout';
}

function renderNtpList() {
  if (!DOM.ntpPanel) return;
  DOM.ntpPanel.innerHTML = NTP_SERVERS.map((server) => {
    const active = server.host === State.ntpServer ? ' active' : '';
    const latency = State.serverLatencies[server.host];
    const rtt = latency ? latency.rtt : -1;
    const status = latency ? latency.status : 'unknown';
    const rttText = rtt > 0 ? `${rtt}ms` : status === 'timeout' ? '超时' : '--';
    const rttClass = getNtpRttClass(rtt);
    return `<div class="ntp-item${active}" data-ntp="${server.host}">
      <span>${server.name} ${server.label}</span>
      <span class="ntp-item-rtt ${rttClass}">${rttText}</span>
    </div>`;
  }).join('');

  DOM.ntpPanel.querySelectorAll('.ntp-item').forEach((element) => {
    element.addEventListener('click', () => setNtp(element.dataset.ntp));
  });
}

function updateWidgetUI() {
  if (!DOM.widgetShell) return;
  const widgetScaleFactor = State.isWidget
    ? Math.max(State.widgetScale / 100, window.innerWidth / WIDGET_BASE_WIDTH)
    : State.widgetScale / 100;
  DOM.widgetShell.style.setProperty('--widget-scale-factor', String(widgetScaleFactor));

  let badgeLabel = '未同步';
  let badgeClass = 'widget-badge idle';

  if (State.hasNtpData && State.isSynced) {
    const tier = State.samples.length >= 3 ? getPrecisionTier() : 'CAL';
    badgeLabel = tier;
    badgeClass = `widget-badge ${tier === 'CAL' ? 'warning' : getPrecisionClass(tier)}`;
  } else if (State.autoSync) {
    badgeLabel = '校准中';
    badgeClass = 'widget-badge warning';
  }

  DOM.widgetBadge.textContent = badgeLabel;
  DOM.widgetBadge.className = badgeClass;
  DOM.widgetTime.className = `widget-time ${State.hasNtpData && State.isSynced && State.samples.length >= 3 ? getPrecisionClass(getPrecisionTier()) : State.autoSync ? 'warning' : 'idle'}`;
  DOM.widgetShell.dataset.tooltip = State.hasNtpData
    ? `${State.sourceLabel}\n偏移 ${State.ntpOffset.toFixed(2)}ms\n延迟 ${State.ntpRtt > 0 ? `${State.ntpRtt}ms` : '--'}\n采样 ${State.samples.length}/20`
    : '未同步\n当前显示本地时间';
}

function applyWidgetScale(scale) {
  State.widgetScale = Number(scale) || 100;
  DOM.widgetScale.value = String(State.widgetScale);
  if (DOM.widgetScaleValue) DOM.widgetScaleValue.textContent = `${State.widgetScale}%`;
  updateWidgetUI();
}

function renderModeUI() {
  if (!DOM.syncMode) return;
  DOM.syncMode.value = State.syncMode;
  DOM.masterHostRow.style.display = State.syncMode === 'slave' ? 'flex' : 'none';
  DOM.modeHint.textContent = MODE_HINTS[State.syncMode] || MODE_HINTS.localNtp;
}

function updateUI() {
  const hasOffsetData = State.hasNtpData && Number.isFinite(State.ntpOffset);
  const offsetText = hasOffsetData ? State.ntpOffset.toFixed(2) : '--';
  DOM.offsetDisplay.textContent = `偏差: ${offsetText}ms`;
  DOM.offsetValue.textContent = offsetText;
  DOM.offsetValue.className = hasOffsetData
    ? 'stat-value ' + cls(Math.abs(State.ntpOffset), 5, 20)
    : 'stat-value';

  if (State.hasNtpData && State.samples.length >= 3) {
    const tier = getPrecisionTier();
    DOM.precisionTier.textContent = tier;
    DOM.precisionTier.className = 'stat-value ' + getPrecisionClass(tier);
    DOM.precisionError.textContent = `±${State.offsetStd.toFixed(2)}ms`;
  } else if (State.hasNtpData) {
    DOM.precisionTier.textContent = '--';
    DOM.precisionTier.className = 'stat-value';
    DOM.precisionError.textContent = `采样 ${State.samples.length}/3`;
  } else {
    DOM.precisionTier.textContent = '--';
    DOM.precisionTier.className = 'stat-value';
    DOM.precisionError.textContent = '--';
  }

  DOM.sampleCount.textContent = String(State.samples.length);

  const activeNtp = NTP_SERVERS.find((server) => server.host === State.ntpServer);
  DOM.ntpName.textContent = activeNtp ? activeNtp.name : State.ntpServer;
  DOM.ntpRttLabel.textContent = State.ntpRtt > 0 ? String(State.ntpRtt) : '--';
  DOM.ntpRttLabel.className = 'stat-value ' + (State.ntpRtt > 0 ? cls(State.ntpRtt, 30, 100) : '');

  const status = getStageStatusText();
  setStatus(status.dot, status.text);
  updateWidgetUI();
}

function renderTzDisplay() {
  const current = TIMEZONES.find((item) => item.value === State.timezone);
  const label = current ? `${current.label} ${current.name}` : State.timezone;
  DOM.tzDisplay.textContent = `时区: ${label} ▾`;
}

function renderTzList() {
  renderTzDisplay();
  DOM.tzPanel.innerHTML = TIMEZONES.map((item) => {
    const active = item.value === State.timezone ? ' active' : '';
    return `<div class="tz-item${active}" data-tz="${item.value}">${item.label} ${item.name}</div>`;
  }).join('');

  DOM.tzPanel.querySelectorAll('.tz-item').forEach((element) => {
    element.addEventListener('click', () => setTz(element.dataset.tz));
  });
}

function initTimezone() {
  const saved = localStorage.getItem('timesync-tz');
  const detected = Intl.DateTimeFormat().resolvedOptions().timeZone;
  State.timezone = saved || detected;
  renderTzList();
}

function setTz(timezone) {
  State.timezone = timezone;
  localStorage.setItem('timesync-tz', timezone);
  DOM.tzPanel.classList.remove('open');
  renderTzList();
  updateUI();
}

function getTzParts(timestamp) {
  const date = new Date(timestamp);
  try {
    const parts = new Intl.DateTimeFormat('en-US', {
      timeZone: State.timezone,
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    }).formatToParts(date);
    return {
      h: parts.find((item) => item.type === 'hour')?.value || '00',
      m: parts.find((item) => item.type === 'minute')?.value || '00',
      s: parts.find((item) => item.type === 'second')?.value || '00'
    };
  } catch (_) {
    return {
      h: String(date.getHours()).padStart(2, '0'),
      m: String(date.getMinutes()).padStart(2, '0'),
      s: String(date.getSeconds()).padStart(2, '0')
    };
  }
}

function animatePulse(element) {
  element.classList.remove('pulse');
  void element.offsetWidth;
  element.classList.add('pulse');
}

function renderLoop() {
  const now = getSyncTime();
  const date = new Date(now);
  const ms = String(date.getMilliseconds()).padStart(3, '0');
  const parts = getTzParts(now);
  const second = parseInt(parts.s, 10);
  const minute = parseInt(parts.m, 10);
  const hour = parseInt(parts.h, 10);

  DOM.hours.textContent = parts.h;
  DOM.mins.textContent = parts.m;
  DOM.secs.textContent = String(second).padStart(2, '0');
  DOM.ms.textContent = `.${ms}`;
  DOM.utcDisplay.textContent = `UTC: ${date.toISOString().replace('T', ' ').substring(0, 23)}`;

  if (second !== lastSec) {
    animatePulse(DOM.secs);
    lastSec = second;
  }
  if (minute !== lastMin) {
    animatePulse(DOM.mins);
    lastMin = minute;
  }
  if (hour !== lastHour) {
    animatePulse(DOM.hours);
    lastHour = hour;
  }

  if (DOM.widgetTime) {
    DOM.widgetTime.textContent = `${parts.h}:${parts.m}:${String(second).padStart(2, '0')}`;
  }

  requestAnimationFrame(renderLoop);
}

function invokeTauri(command, args) {
  if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function') {
    return window.__TAURI_INTERNALS__.invoke(command, args);
  }
  return Promise.reject(new Error('No Tauri IPC'));
}

function openUpdateModal(version, notes) {
  DOM.updateCurrentVersion.textContent = State.appVersion ? `v${State.appVersion}` : '--';
  DOM.updateLatestVersion.textContent = version ? `v${version}` : '--';
  DOM.updateNotes.textContent = notes || '暂无版本说明';
  DOM.updateModal.classList.add('open');
}

function closeUpdateModal() {
  DOM.updateModal.classList.remove('open');
}

function formatByteProgress(value) {
  if (!value || value <= 0) return '0 B';
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

function renderUpdateProgress(status) {
  const downloaded = status?.downloadedBytes || 0;
  const total = status?.totalBytes || 0;
  const percent = total > 0 ? clamp(Math.round(downloaded / total * 100), 0, 100) : 0;
  DOM.updateProgressBar.style.width = `${percent}%`;

  if (status?.phase === 'downloaded') {
    DOM.updateProgressBar.style.width = '100%';
    DOM.updateProgressText.textContent = '下载完成';
    return;
  }
  if (status?.phase === 'installing') {
    DOM.updateProgressText.textContent = '正在启动安装器，安装完成后应用会自动退出并回到新版本';
    return;
  }
  if (status?.phase === 'downloading') {
    if (total > 0) {
      DOM.updateProgressText.textContent = `${percent}% · ${formatByteProgress(downloaded)} / ${formatByteProgress(total)}`;
    } else {
      DOM.updateProgressText.textContent = `已下载 ${formatByteProgress(downloaded)}`;
    }
    return;
  }
  DOM.updateProgressText.textContent = status?.message || '等待开始';
  DOM.updateProgressBar.style.width = ['available', 'idle', 'upToDate', 'error'].includes(status?.phase) ? '0%' : DOM.updateProgressBar.style.width;
}

function updateStatusClass(phase) {
  if (phase === 'checking') return 'update-status checking';
  if (phase === 'available' || phase === 'downloaded') return 'update-status ready';
  if (phase === 'downloading' || phase === 'installing') return 'update-status downloading';
  if (phase === 'error') return 'update-status error';
  return 'update-status';
}

function renderUpdateStatus(status) {
  if (!status) return;
  State.currentUpdatePhase = status.phase || 'idle';
  DOM.updateStatus.textContent = status.message || '';
  DOM.updateStatus.className = updateStatusClass(status.phase);
  renderUpdateProgress(status);

  if (status.phase === 'checking') {
    if (!DOM.updateModal.classList.contains('open')) {
      openUpdateModal(status.version, status.notes);
    }
    DOM.btnCheckUpdate.disabled = true;
    DOM.btnCheckUpdate.textContent = '检查中...';
    return;
  }

  if (status.phase === 'available') {
    DOM.btnCheckUpdate.disabled = false;
    DOM.btnCheckUpdate.textContent = '下载更新';
    openUpdateModal(status.version, status.notes);
    return;
  }

  if (status.phase === 'downloading') {
    openUpdateModal(status.version, status.notes);
    DOM.btnCheckUpdate.disabled = true;
    DOM.btnCheckUpdate.textContent = '下载中...';
    return;
  }

  if (status.phase === 'downloaded') {
    openUpdateModal(status.version, status.notes);
    DOM.btnCheckUpdate.disabled = false;
    DOM.btnCheckUpdate.textContent = '安装并重启';
    return;
  }

  if (status.phase === 'installing') {
    openUpdateModal(status.version, status.notes);
    DOM.btnCheckUpdate.disabled = true;
    DOM.btnCheckUpdate.textContent = '即将重启...';
    DOM.btnCheckUpdate.classList.add('installing');
    return;
  }

  DOM.btnCheckUpdate.disabled = false;
  DOM.btnCheckUpdate.textContent = '检查更新';
  DOM.btnCheckUpdate.classList.remove('installing');
  if (status.phase !== 'available') {
    closeUpdateModal();
  }
}

async function pollUpdateStatus() {
  try {
    const status = await invokeTauri('get_update_status');
    renderUpdateStatus(status);
    if (!['checking', 'downloading', 'installing'].includes(status.phase)) {
      stopUpdatePolling();
    }
  } catch (error) {
    stopUpdatePolling();
    DOM.updateStatus.textContent = '获取更新状态失败：' + (error.message || error);
    DOM.updateStatus.className = 'update-status error';
    DOM.btnCheckUpdate.disabled = false;
    DOM.btnCheckUpdate.textContent = '检查更新';
    DOM.btnCheckUpdate.classList.remove('installing');
  }
}

function startUpdatePolling() {
  stopUpdatePolling();
  pollUpdateStatus();
  updatePollTimer = setInterval(pollUpdateStatus, 500);
}

function stopUpdatePolling() {
  if (updatePollTimer) {
    clearInterval(updatePollTimer);
    updatePollTimer = 0;
  }
}

async function doCheckUpdate() {
  renderUpdateStatus({ phase: 'checking', message: '正在检查更新...' });
  try {
    const status = await invokeTauri('check_for_update');
    renderUpdateStatus(status);
  } catch (error) {
    renderUpdateStatus({
      phase: 'error',
      message: '检查失败：' + (error.message || error)
    });
  }
}

async function startDownloadUpdate() {
  DOM.updateStatus.textContent = '正在准备下载更新...';
  DOM.updateStatus.className = 'update-status downloading';
  DOM.btnCheckUpdate.disabled = true;
  DOM.btnCheckUpdate.textContent = '下载中...';
  try {
    await invokeTauri('download_available_update');
    startUpdatePolling();
  } catch (error) {
    renderUpdateStatus({
      phase: 'error',
      message: '启动下载失败：' + (error.message || error)
    });
  }
}

async function startInstallUpdate() {
  DOM.updateStatus.textContent = '正在准备安装，应用将自动退出并由安装器完成更新...';
  DOM.updateStatus.className = 'update-status downloading';
  DOM.btnCheckUpdate.disabled = true;
  DOM.btnCheckUpdate.textContent = '即将重启...';
  try {
    await new Promise((resolve) => setTimeout(resolve, 250));
    await invokeTauri('install_downloaded_update');
    startUpdatePolling();
  } catch (error) {
    renderUpdateStatus({
      phase: 'error',
      message: '启动安装失败：' + (error.message || error)
    });
  }
}

async function pollOnce() {
  try {
    const payload = await invokeTauri('get_ntp_status');
    if (payload) {
      handleTime(payload);
      return true;
    }
  } catch (error) {
    dbg('poll error: ' + error.message);
  }
  return false;
}

function startNtpPolling() {
  ntpPollTimer = setInterval(pollOnce, 1000);
}

function stopNtpPolling() {
  if (ntpPollTimer) {
    clearInterval(ntpPollTimer);
    ntpPollTimer = 0;
  }
}

async function refreshWidgetSettings() {
  if (!State.isWidget) return;
  try {
    const settings = await invokeTauri('get_sync_settings');
    if (Number(settings.widgetScale) !== State.widgetScale) {
      applyWidgetScale(settings.widgetScale);
    }
    State.widgetEnabled = !!settings.widgetEnabled;
  } catch (_) {}
}

function startWidgetSettingsPolling() {
  if (!State.isWidget) return;
  stopWidgetSettingsPolling();
  refreshWidgetSettings();
  widgetSettingsPollTimer = setInterval(refreshWidgetSettings, 300);
}

function stopWidgetSettingsPolling() {
  if (widgetSettingsPollTimer) {
    clearInterval(widgetSettingsPollTimer);
    widgetSettingsPollTimer = 0;
  }
}

function renderBrowserMode() {
  setStatus('error', 'BROWSER MODE');
  DOM.titlebar?.remove();
  DOM.versionNum.textContent = 'mock';
  State.appVersion = 'mock';
  renderModeUI();
  updateUI();
  renderBootState();
  setTimeout(() => hideBootOverlay(), 900);
}

async function loadRuntimeContext() {
  const context = await invokeTauri('get_runtime_context');
  State.isWidget = !!context.isWidget;
  State.appVersion = context.version || '';
  if (State.appVersion) {
    DOM.versionNum.textContent = 'v' + State.appVersion;
  }
  DOM.body.classList.toggle('widget-mode', State.isWidget);
  if (State.isWidget) {
    hideBootOverlay();
  }
}

async function loadSyncSettings() {
  const settings = await invokeTauri('get_sync_settings');
  State.autoSync = !!settings.autoSync;
  State.syncIntervalSecs = Number(settings.syncIntervalSecs) || 5;
  State.syncMode = settings.syncMode || 'localNtp';
  State.masterHost = settings.masterHost || '127.0.0.1:36363';
  State.pairCode = settings.pairCode || '';
  State.widgetEnabled = !!settings.widgetEnabled;
  State.calibrationStage = settings.calibrationStage || 'calibrating';
  State.activeServers = settings.activeServers || [];
  State.ntpServer = settings.ntpServer || State.ntpServer;

  DOM.chkAutoSync.checked = State.autoSync;
  DOM.syncInterval.value = String(State.syncIntervalSecs);
  applyWidgetScale(settings.widgetScale);
  DOM.masterHost.value = State.masterHost;
  DOM.pairCode.value = State.pairCode;
  DOM.chkWidgetEnabled.checked = State.widgetEnabled;
  renderModeUI();
  renderNtpList();
  updateUI();
  renderBootState();
  maybeFinishBootOverlay(120);
}

function setupTitlebar() {
  if (State.isWidget) return;
  if (!window.__TAURI_INTERNALS__) return;

  const dragRegion = document.getElementById('titlebarDragRegion');
  const btnMinimize = document.getElementById('btnMinimize');
  const btnMaximize = document.getElementById('btnMaximize');
  const btnClose = document.getElementById('btnClose');
  let pendingDrag = null;

  btnMinimize.onclick = () => invokeTauri('minimize_window');
  btnMaximize.onclick = () => invokeTauri('maximize_window');
  btnClose.onclick = () => invokeTauri('close_window');

  dragRegion?.addEventListener('mousedown', (event) => {
    if (event.button !== 0) return;
    if (event.target.closest('.titlebar-controls, .tb-btn, .settings-panel')) return;
    if (event.detail > 1) {
      pendingDrag = null;
      return;
    }
    pendingDrag = { x: event.clientX, y: event.clientY };
  });

  document.addEventListener('mousemove', (event) => {
    if (!pendingDrag) return;
    if ((event.buttons & 1) !== 1) {
      pendingDrag = null;
      return;
    }
    const dx = Math.abs(event.clientX - pendingDrag.x);
    const dy = Math.abs(event.clientY - pendingDrag.y);
    if (dx < 3 && dy < 3) return;
    pendingDrag = null;
    invokeTauri('start_drag').catch(() => {});
  });

  document.addEventListener('mouseup', () => {
    pendingDrag = null;
  });

  window.addEventListener('blur', () => {
    pendingDrag = null;
  });

  dragRegion?.addEventListener('dblclick', (event) => {
    if (event.target.closest('.titlebar-controls, .tb-btn, .settings-panel')) return;
    pendingDrag = null;
    invokeTauri('maximize_window').catch(() => {});
  });
}

function setupInteractions() {
  document.addEventListener('click', (event) => {
    if (!event.target.closest('.tz-selector')) {
      DOM.tzPanel.classList.remove('open');
    }
    if (!event.target.closest('.ntp-selector') && !event.target.closest('.ntp-panel')) {
      DOM.ntpPanel.classList.remove('open');
    }
    if (event.target.closest('.tz-label')) {
      DOM.tzPanel.classList.toggle('open');
      if (DOM.tzPanel.classList.contains('open')) renderTzList();
    }
    if (event.target.closest('.ntp-selector')) {
      DOM.ntpPanel.classList.toggle('open');
      if (DOM.ntpPanel.classList.contains('open')) renderNtpList();
    }
    if (event.target.closest('#btnSettings')) {
      DOM.settingsPanel.classList.toggle('open');
    } else if (!event.target.closest('.settings-panel')) {
      DOM.settingsPanel.classList.remove('open');
    }
  });

  DOM.btnSyncNtpNow.addEventListener('click', async () => {
    DOM.btnSyncNtpNow.disabled = true;
    DOM.syncStatus.textContent = '正在同步NTP...';
    try {
      const result = await invokeTauri('sync_ntp_now');
      await pollOnce();
      DOM.syncStatus.textContent = result;
      DOM.syncStatus.style.color = 'var(--green)';
    } catch (error) {
      await pollOnce().catch(() => {});
      DOM.syncStatus.textContent = error.message || 'NTP同步失败';
      DOM.syncStatus.style.color = 'var(--red)';
    }
    DOM.btnSyncNtpNow.disabled = false;
    setTimeout(() => {
      DOM.syncStatus.textContent = '';
    }, 5000);
  });

  DOM.btnSyncNow.addEventListener('click', async () => {
    DOM.btnSyncNow.disabled = true;
    DOM.syncStatus.textContent = '正在同步系统时间...';
    try {
      const result = await invokeTauri('sync_system_time');
      DOM.syncStatus.textContent = result;
      DOM.syncStatus.style.color = 'var(--green)';
    } catch (error) {
      DOM.syncStatus.textContent = error.message || '同步失败';
      DOM.syncStatus.style.color = 'var(--red)';
    }
    DOM.btnSyncNow.disabled = false;
    setTimeout(() => {
      DOM.syncStatus.textContent = '';
    }, 5000);
  });

  DOM.chkAutoSync.addEventListener('change', async () => {
    State.autoSync = DOM.chkAutoSync.checked;
    if (State.autoSync) {
      resetSyncState();
    } else {
      State.calibrationStage = 'idle';
    }
    updateUI();
    try {
      await invokeTauri('set_auto_sync', { enabled: State.autoSync });
      if (State.autoSync) {
        await invokeTauri('sync_ntp_now');
        await pollOnce();
      }
    } catch (_) {}
  });

  DOM.syncInterval.addEventListener('change', () => {
    const seconds = clamp(parseInt(DOM.syncInterval.value, 10) || 5, 2, 3600);
    DOM.syncInterval.value = String(seconds);
    State.syncIntervalSecs = seconds;
    invokeTauri('set_sync_interval', { seconds }).catch(() => {});
  });

  DOM.syncMode.addEventListener('change', async () => {
    State.syncMode = DOM.syncMode.value;
    resetSyncState();
    renderModeUI();
    updateUI();
    try {
      await invokeTauri('set_sync_mode', { mode: State.syncMode });
      if (State.autoSync) {
        await invokeTauri('sync_ntp_now');
        await pollOnce();
      }
    } catch (_) {}
  });

  DOM.masterHost.addEventListener('change', () => {
    State.masterHost = DOM.masterHost.value.trim();
    invokeTauri('set_master_host', { host: State.masterHost }).catch(() => {});
  });

  DOM.pairCode.addEventListener('change', () => {
    State.pairCode = DOM.pairCode.value.trim();
    invokeTauri('set_pair_code', { code: State.pairCode }).catch(() => {});
  });

  DOM.chkWidgetEnabled.addEventListener('change', () => {
    State.widgetEnabled = DOM.chkWidgetEnabled.checked;
    invokeTauri('set_widget_enabled', { enabled: State.widgetEnabled }).catch(() => {});
  });

  DOM.widgetScale?.addEventListener('input', () => {
    const scale = clamp(parseInt(DOM.widgetScale.value, 10) || 100, 80, 220);
    DOM.widgetScale.value = String(scale);
    applyWidgetScale(scale);
    if (widgetScalePersistTimer) clearTimeout(widgetScalePersistTimer);
    widgetScalePersistTimer = setTimeout(() => {
      invokeTauri('set_widget_scale', { scale }).catch(() => {});
      widgetScalePersistTimer = 0;
    }, 120);
  });

  DOM.widgetScale?.addEventListener('change', () => {
    const scale = clamp(parseInt(DOM.widgetScale.value, 10) || 100, 80, 220);
    DOM.widgetScale.value = String(scale);
    applyWidgetScale(scale);
    invokeTauri('set_widget_scale', { scale }).catch(() => {});
  });

  DOM.btnCheckUpdate.addEventListener('click', () => {
    if (State.currentUpdatePhase === 'available') {
      startDownloadUpdate();
    } else if (State.currentUpdatePhase === 'downloaded') {
      startInstallUpdate();
    } else {
      doCheckUpdate();
    }
  });

  DOM.btnUpdateLater.addEventListener('click', closeUpdateModal);
  DOM.btnUpdateNow.addEventListener('click', () => {
    if (State.currentUpdatePhase === 'downloaded') {
      startInstallUpdate();
    } else {
      startDownloadUpdate();
    }
  });

  DOM.widgetShell?.addEventListener('dblclick', async (event) => {
    if (!State.isWidget) return;
    event.stopPropagation();
    if (event.target === DOM.widgetClose) return;
    widgetClickGuardUntil = Date.now() + 350;
    try {
      await invokeTauri('restore_main_window');
    } catch (_) {}
  });

  DOM.widgetClose?.addEventListener('click', async (event) => {
    event.stopPropagation();
    event.preventDefault();
    try {
      State.widgetEnabled = false;
      DOM.chkWidgetEnabled.checked = false;
      await invokeTauri('dismiss_widget');
      updateUI();
    } catch (_) {}
  });

  DOM.widgetShell?.addEventListener('mousedown', (event) => {
    if (!State.isWidget || event.button !== 0) return;
    if (event.target === DOM.widgetClose) return;
    widgetDrag = {
      startX: event.clientX,
      startY: event.clientY,
      dragging: false
    };
    DOM.widgetShell.classList.add('dragging');
  });

  document.addEventListener('mousemove', (event) => {
    if (!State.isWidget || !widgetDrag) return;
    const dx = event.clientX - widgetDrag.startX;
    const dy = event.clientY - widgetDrag.startY;
    if (Math.abs(dx) < 3 && Math.abs(dy) < 3) return;
    widgetDrag.dragging = true;
    widgetDrag = null;
    widgetClickGuardUntil = Date.now() + 350;
    invokeTauri('start_widget_drag').catch(() => {});
  });

  document.addEventListener('mouseup', () => {
    if (!State.isWidget) return;
    DOM.widgetShell?.classList.remove('dragging');
    if (!widgetDrag) {
      setTimeout(() => {
        invokeTauri('save_widget_position').catch(() => {});
      }, 120);
    }
    widgetDrag = null;
  });
}

function setNtp(host) {
  invokeTauri('set_ntp_server', { server: host })
    .then(() => {
      State.ntpServer = host;
      resetSyncState();
      renderNtpList();
      updateUI();
      return invokeTauri('sync_ntp_now');
    })
    .then(() => pollOnce())
    .catch(() => {});
  DOM.ntpPanel.classList.remove('open');
}

function cleanup() {
  stopNtpPolling();
  stopWidgetSettingsPolling();
  stopUpdatePolling();
  if (bootTimer) {
    clearTimeout(bootTimer);
    bootTimer = 0;
  }
}

async function initApp() {
  dbg('App init starting');
  State.bootStartedAt = Date.now();
  renderBootState();
  initTimezone();
  setupInteractions();
  bootTimer = setTimeout(() => {
    renderBootState();
    maybeFinishBootOverlay();
  }, 35000);

  const hasTauri = !!(window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function');
  if (!hasTauri) {
    renderBrowserMode();
    requestAnimationFrame(renderLoop);
    return;
  }

  try {
    await loadRuntimeContext();
    setupTitlebar();
    await loadSyncSettings();
    renderBootState();
    const updateStatus = await invokeTauri('get_update_status');
    renderUpdateStatus(updateStatus);
    startWidgetSettingsPolling();
    startNtpPolling();
    await pollOnce();
    renderBootState();
    maybeFinishBootOverlay(120);
  } catch (error) {
    dbg('init error: ' + (error.message || error));
    setStatus('error', '初始化失败');
    renderBootState();
    setTimeout(() => hideBootOverlay(), 900);
  }

  requestAnimationFrame(renderLoop);
}

window.addEventListener('beforeunload', cleanup);
window.addEventListener('resize', () => {
  if (State.isWidget) {
    updateWidgetUI();
  }
});
initApp();
