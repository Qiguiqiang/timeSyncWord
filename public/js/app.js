function dbg(msg) { console.log('[TS]', msg); }

const DOM = {
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
  btnSyncNow: document.getElementById('btnSyncNow'),
  chkAutoSync: document.getElementById('chkAutoSync'),
  syncInterval: document.getElementById('syncInterval'),
  syncStatus: document.getElementById('syncStatus')
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

const State = {
  offset: 0,
  samples: [],
  maxSamples: 20,
  isSynced: false,
  syncBase: 0,
  perfBase: 0,
  offsetStd: 0,
  ntpOffset: 0,
  timezone: '',
  ntpServer: 'ntp.tencent.com',
  ntpRtt: -1,
  serverLatencies: {}
};

function calculateStdDev(arr) {
  if (arr.length < 2) return 0;
  const mean = arr.reduce((a, b) => a + b, 0) / arr.length;
  const variance = arr.reduce((sum, v) => sum + Math.pow(v - mean, 2), 0) / arr.length;
  return Math.sqrt(variance);
}

function getPrecisionTier() {
  const { offsetStd } = State;
  if (offsetStd < 2)   return 'S+';
  if (offsetStd < 5)   return 'S';
  if (offsetStd < 10)  return 'S-';
  if (offsetStd < 30)  return 'A';
  if (offsetStd < 50)  return 'B';
  if (offsetStd < 100) return 'C';
  return 'D';
}

function getPrecisionClass(tier) {
  if (tier === 'S+' || tier === 'S') return 'good';
  if (tier === 'S-' || tier === 'A') return 'warning';
  return 'danger';
}

function handleTime(msg) {
  if (!msg || typeof msg.ntp_offset === 'undefined') return;
  const offset = msg.ntp_offset;

  State.samples.push(offset);
  if (State.samples.length > State.maxSamples) State.samples.shift();

  State.ntpOffset = offset;
  if (msg.ntp_server) State.ntpServer = msg.ntp_server;
  if (msg.ntp_rtt !== undefined) State.ntpRtt = msg.ntp_rtt;
  if (msg.server_latencies) State.serverLatencies = msg.server_latencies;

  calculateOffset();
  updateUI();
}

function calculateOffset() {
  if (State.samples.length < 3) { State.isSynced = false; return; }

  const sorted = [...State.samples].sort((a, b) => a - b);
  const cutoff = Math.floor(sorted.length * 0.1);
  const filtered = sorted.slice(cutoff, sorted.length - cutoff);

  State.offset = filtered.reduce((a, b) => a + b, 0) / filtered.length;
  State.offsetStd = calculateStdDev(filtered);

  State.syncBase = Date.now() + State.offset;
  State.perfBase = performance.now();
  State.isSynced = true;
}

function getSyncTime() { return State.syncBase + (performance.now() - State.perfBase); }

function updateUI() {
  const offsetText = State.ntpOffset.toFixed(2);
  DOM.offsetDisplay.textContent = `偏差: ${offsetText}ms`;

  const tier = getPrecisionTier();
  DOM.precisionTier.textContent = tier;
  DOM.precisionTier.className = 'stat-value ' + getPrecisionClass(tier);
  DOM.precisionError.textContent = `±${State.offsetStd.toFixed(2)}ms`;
  DOM.sampleCount.textContent = State.samples.length;

  DOM.offsetValue.textContent = offsetText;
  DOM.offsetValue.className = 'stat-value ' + cls(Math.abs(State.ntpOffset), 5, 20);

  setStatus('synced', `已同步 偏移 ${State.ntpOffset.toFixed(1)}ms 精度 ±${State.offsetStd.toFixed(1)}ms`);

  const activeNtp = NTP_SERVERS.find(s => s.host === State.ntpServer);
  const ntpLabel = activeNtp ? activeNtp.name : State.ntpServer;
  DOM.ntpRttLabel.textContent = State.ntpRtt > 0 ? State.ntpRtt : '--';
  DOM.ntpRttLabel.className = 'stat-value ' + (State.ntpRtt > 0 ? cls(State.ntpRtt, 30, 100) : '');
  DOM.ntpName.textContent = ntpLabel;
}

function getNtpRttClass(rtt) {
  if (rtt <= 0) return '';
  if (rtt < 30) return 'ok';
  if (rtt < 100) return 'warning';
  return 'timeout';
}

function handleNtpChange(msg) {
  if (msg.ntp_server) State.ntpServer = msg.ntp_server;
  if (msg.ntp_rtt !== undefined) State.ntpRtt = msg.ntp_rtt;
  if (msg.server_latencies) State.serverLatencies = msg.server_latencies;
  renderNtpList();
}

function setNtp(host) {
  invokeTauri('set_ntp_server', { server: host }).then(() => {
    State.ntpServer = host;
    State.samples = [];
    renderNtpList();
  }).catch(() => {});
  DOM.ntpPanel.classList.remove('open');
}

function renderNtpList() {
  DOM.ntpPanel.innerHTML = NTP_SERVERS.map(s => {
    const active = s.host === State.ntpServer ? ' active' : '';
    const latency = State.serverLatencies[s.host];
    const rtt = latency ? latency.rtt : -1;
    const status = latency ? latency.status : 'unknown';
    const rttText = rtt > 0 ? `${rtt}ms` : status === 'timeout' ? '超时' : '--';
    const rttClass = getNtpRttClass(rtt);
    return `<div class="ntp-item${active}" data-ntp="${s.host}">
      <span>${s.name} ${s.label}</span>
      <span class="ntp-item-rtt ${rttClass}">${rttText}</span>
    </div>`;
  }).join('');

  DOM.ntpPanel.querySelectorAll('.ntp-item').forEach(el => {
    el.addEventListener('click', () => setNtp(el.dataset.ntp));
  });
}

function cls(v, warn, danger) { return v < warn ? 'good' : v < danger ? 'warning' : 'danger'; }

function setStatus(state, label) {
  DOM.statusDot.className = 'status-dot ' + state;
  DOM.statusLabel.textContent = label;
}

function initTimezone() {
  const saved = localStorage.getItem('timesync-tz');
  const detected = Intl.DateTimeFormat().resolvedOptions().timeZone;
  State.timezone = saved || detected;
  renderTzList();
}

function setTz(tz) {
  State.timezone = tz;
  localStorage.setItem('timesync-tz', tz);
  DOM.tzPanel.classList.remove('open');
  renderTzList();
  State.syncBase = Date.now() + State.offset;
  State.perfBase = performance.now();
}

function renderTzList() {
  renderTzDisplay();
  DOM.tzPanel.innerHTML = TIMEZONES.map(t => {
    const active = t.value === State.timezone ? ' active' : '';
    return `<div class="tz-item${active}" data-tz="${t.value}">${t.label} ${t.name}</div>`;
  }).join('');

  DOM.tzPanel.querySelectorAll('.tz-item').forEach(el => {
    el.addEventListener('click', () => setTz(el.dataset.tz));
  });
}

function renderTzDisplay() {
  const current = TIMEZONES.find(t => t.value === State.timezone);
  const label = current ? `${current.label} ${current.name}` : State.timezone;
  DOM.tzDisplay.textContent = `时区: ${label} ▾`;
}

document.addEventListener('click', (e) => {
  if (!e.target.closest('.tz-selector')) {
    DOM.tzPanel.classList.remove('open');
  }
  if (!e.target.closest('.ntp-selector') && !e.target.closest('.ntp-panel')) {
    DOM.ntpPanel.classList.remove('open');
  }
});

document.addEventListener('click', (e) => {
  if (e.target.closest('.tz-label')) {
    DOM.tzPanel.classList.toggle('open');
    if (DOM.tzPanel.classList.contains('open')) renderTzList();
  }
});

document.addEventListener('click', (e) => {
  if (e.target.closest('.ntp-selector')) {
    DOM.ntpPanel.classList.toggle('open');
    if (DOM.ntpPanel.classList.contains('open')) renderNtpList();
  }
});

document.addEventListener('click', (e) => {
  if (e.target.closest('#btnSettings')) {
    DOM.settingsPanel.classList.toggle('open');
  } else if (!e.target.closest('.settings-panel')) {
    DOM.settingsPanel.classList.remove('open');
  }
});

DOM.btnSyncNow.addEventListener('click', async () => {
  DOM.btnSyncNow.disabled = true;
  DOM.syncStatus.textContent = '同步中...';
  try {
    const res = await invokeTauri('sync_system_time');
    DOM.syncStatus.textContent = res;
    DOM.syncStatus.style.color = 'var(--green)';
  } catch (e) {
    DOM.syncStatus.textContent = e.message || '同步失败';
    DOM.syncStatus.style.color = 'var(--red)';
  }
  DOM.btnSyncNow.disabled = false;
  setTimeout(() => { DOM.syncStatus.textContent = ''; }, 5000);
});

DOM.chkAutoSync.addEventListener('change', () => {
  invokeTauri('set_auto_sync', { enabled: DOM.chkAutoSync.checked }).catch(() => {});
});

DOM.syncInterval.addEventListener('change', () => {
  const val = parseInt(DOM.syncInterval.value) || 30;
  DOM.syncInterval.value = Math.max(5, Math.min(3600, val));
  invokeTauri('set_sync_interval', { seconds: DOM.syncInterval.value }).catch(() => {});
});

let lastSec = -1, lastMin = -1, lastHour = -1;
function animatePulse(el) {
  el.classList.remove('pulse');
  void el.offsetWidth;
  el.classList.add('pulse');
}
function renderLoop() {
  if (State.isSynced) {
    const now = getSyncTime();
    const d = new Date(now);
    const ms = String(d.getMilliseconds()).padStart(3, '0');

    const tzParts = getTzParts(now);
    const hStr = tzParts.h, mStr = tzParts.m;
    const sec = parseInt(tzParts.s);
    const min = parseInt(mStr);
    const hour = parseInt(hStr);

    DOM.hours.textContent = hStr;
    DOM.mins.textContent = mStr;
    DOM.secs.textContent = String(sec).padStart(2, '0');

    if (sec !== lastSec) { animatePulse(DOM.secs); lastSec = sec; }
    if (min !== lastMin) { animatePulse(DOM.mins); lastMin = min; }
    if (hour !== lastHour) { animatePulse(DOM.hours); lastHour = hour; }

    DOM.ms.textContent = `.${ms}`;

    DOM.utcDisplay.textContent = `UTC: ${d.toISOString().replace('T', ' ').substring(0, 23)}`;
  }
  requestAnimationFrame(renderLoop);
}

function getTzParts(timestamp) {
  const d = new Date(timestamp);
  try {
    const parts = new Intl.DateTimeFormat('en-US', {
      timeZone: State.timezone,
      hour12: false,
      hour: '2-digit', minute: '2-digit', second: '2-digit'
    }).formatToParts(d);
    const h = parts.find(p => p.type === 'hour')?.value || '00';
    const m = parts.find(p => p.type === 'minute')?.value || '00';
    const s = parts.find(p => p.type === 'second')?.value || '00';
    return { h, m, s };
  } catch {
    return {
      h: String(d.getHours()).padStart(2, '0'),
      m: String(d.getMinutes()).padStart(2, '0'),
      s: String(d.getSeconds()).padStart(2, '0')
    };
  }
}

function invokeTauri(cmd, args) {
  if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
    return window.__TAURI_INTERNALS__.invoke(cmd, args);
  }
  console.error('invokeTauri: No __TAURI_INTERNALS__.invoke for', cmd);
  return Promise.reject(new Error('No Tauri IPC'));
}

function setupTitlebar() {
  if (window.__TAURI_INTERNALS__) {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    document.getElementById('btnMinimize').onclick = () => invoke('minimize_window');
    document.getElementById('btnMaximize').onclick = () => invoke('maximize_window');
    document.getElementById('btnClose').onclick = () => invoke('close_window');
  } else {
    document.querySelector('.titlebar')?.remove();
  }
}

let ntpPollTimer = 0;
async function pollOnce() {
  try {
    const msg = await invokeTauri('get_ntp_status');
    if (msg) {
      dbg('NTP data: offset=' + msg.ntp_offset?.toFixed(2) + ' rtt=' + msg.ntp_rtt + ' svr=' + msg.ntp_server);
      handleTime(msg);
      return true;
    } else {
      dbg('invoke OK but null data');
    }
  } catch (e) {
    dbg('poll err: ' + e.message);
  }
  return false;
}

function startNtpPolling() {
  pollOnce().then(ok => {
    if (!ok) dbg('First poll failed, will retry in 2s');
  });
  ntpPollTimer = setInterval(pollOnce, 2000);
}

function testInvoke(cmd) {
  invokeTauri(cmd || 'get_ntp_status').then(r => {
    dbg(cmd + ': OK ' + JSON.stringify(r).substring(0, 60));
  }).catch(e => {
    dbg(cmd + ': FAIL ' + e.message);
  });
}

function cleanup() {
  if (ntpPollTimer) clearInterval(ntpPollTimer);
}

(function init() {
  dbg('App init starting...');
  const internals = window.__TAURI_INTERNALS__;
  dbg('__TAURI_INTERNALS__ = ' + (internals ? 'YES' : 'NO'));
  if (internals) {
    dbg('invoke type = ' + (typeof internals.invoke));
    dbg('listen type = ' + (typeof internals.listen));
    dbg('invoke keys = ' + Object.keys(internals).join(', '));
  }

  if (internals && typeof internals.invoke === 'function') {
    setStatus('synced', 'Tauri IPC OK');
    setupTitlebar();
    initTimezone();
    startNtpPolling();
    setTimeout(() => testInvoke('ping'), 500);
    setTimeout(() => testInvoke('get_cycle_count'), 1000);
    setTimeout(() => testInvoke('get_ntp_status'), 1500);
    invokeTauri('get_auto_sync').then(v => { DOM.chkAutoSync.checked = v; }).catch(() => {});
    invokeTauri('get_sync_interval').then(v => { DOM.syncInterval.value = v; }).catch(() => {});
  } else {
    setStatus('danger', 'BROWSER MODE');
    document.querySelector('.titlebar')?.remove();
    initTimezone();
  }
  requestAnimationFrame(renderLoop);
})();

window.addEventListener('beforeunload', cleanup);
