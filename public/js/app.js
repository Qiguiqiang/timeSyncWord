const DOM = {
  currentTime: document.getElementById('currentTime'),
  currentMs: document.getElementById('currentMs'),
  utcDisplay: document.getElementById('utcDisplay'),
  offsetDisplay: document.getElementById('offsetDisplay'),
  rttDisplay: document.getElementById('rttDisplay'),
  offsetValue: document.getElementById('offsetValue'),
  rttValue: document.getElementById('rttValue'),
  precisionDisplay: document.getElementById('precisionDisplay'),
  sampleCount: document.getElementById('sampleCount'),
  statusDot: document.getElementById('statusDot'),
  statusLabel: document.getElementById('statusLabel')
};

const State = {
  ws: null,
  offset: 0,
  samples: [],
  maxSamples: 20,
  isSynced: false
};

function connect() {
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
  State.ws = new WebSocket(`${proto}//${location.host}`);

  State.ws.onopen = () => {
    setStatus('connecting', 'CONNECTED');
    startSync();
  };

  State.ws.onmessage = (e) => {
    const msg = JSON.parse(e.data);
    if (msg.type === 'time') handleTime(msg);
  };

  State.ws.onclose = () => {
    setStatus('error', 'DISCONNECTED');
    stopSync();
    setTimeout(connect, 3000);
  };

  State.ws.onerror = () => {};
}

function send(data) {
  if (State.ws && State.ws.readyState === WebSocket.OPEN) {
    State.ws.send(JSON.stringify(data));
  }
}

function handleTime(msg) {
  const T1 = msg.t1, T2 = msg.serverTime, T3 = Date.now();
  const rtt = T3 - T1;
  const offset = T2 - (T1 + T3) / 2;

  State.samples.push({ offset, rtt });
  if (State.samples.length > State.maxSamples) State.samples.shift();

  calculateOffset();
  updateUI(rtt, offset);
}

function calculateOffset() {
  if (State.samples.length < 3) { State.isSynced = false; return; }
  const sorted = [...State.samples].sort((a, b) => a.rtt - b.rtt);
  const cutoff = Math.floor(sorted.length * 0.1);
  const filtered = sorted.slice(cutoff, sorted.length - cutoff);
  let tw = 0, ws = 0;
  for (const s of filtered) { const w = 1 / (s.rtt + 1); tw += w; ws += s.offset * w; }
  State.offset = ws / tw;
  State.isSynced = true;
}

function getSyncTime() { return Date.now() + State.offset; }

function updateUI(rtt, offset) {
  const now = getSyncTime();
  const d = new Date(now);
  const h = String(d.getHours()).padStart(2, '0');
  const m = String(d.getMinutes()).padStart(2, '0');
  const s = String(d.getSeconds()).padStart(2, '0');
  const ms = String(d.getMilliseconds()).padStart(3, '0');

  DOM.currentTime.textContent = `${h}:${m}:${s}`;
  DOM.currentMs.textContent = `.${ms}`;
  DOM.utcDisplay.textContent = `UTC: ${d.toISOString().replace('T', ' ').substring(0, 23)}`;

  const abs = Math.abs(State.offset);
  DOM.offsetDisplay.textContent = `OFFSET: ${State.offset.toFixed(2)}ms`;
  DOM.rttDisplay.textContent = `RTT: ${rtt}ms`;

  DOM.offsetValue.textContent = State.offset.toFixed(2);
  DOM.offsetValue.className = 'stat-value ' + cls(abs, 5, 20);
  DOM.rttValue.textContent = rtt;
  DOM.rttValue.className = 'stat-value ' + cls(rtt, 50, 100);

  const tier = abs < 5 ? 'S+' : abs < 10 ? 'S' : abs < 30 ? 'A' : abs < 50 ? 'B' : 'C';
  DOM.precisionDisplay.textContent = tier;
  DOM.precisionDisplay.className = 'stat-value ' + (abs < 10 ? 'good' : abs < 30 ? 'warning' : 'danger');
  DOM.sampleCount.textContent = State.samples.length;

  setStatus('synced', `SYNCED ±${State.offset.toFixed(1)}ms`);
}

function cls(v, warn, danger) { return v < warn ? 'good' : v < danger ? 'warning' : 'danger'; }

function setStatus(state, label) {
  DOM.statusDot.className = 'status-dot ' + state;
  DOM.statusLabel.textContent = label;
}

function startSync() {
  function loop() {
    if (State.ws && State.ws.readyState === WebSocket.OPEN) {
      send({ type: 'getTime', t1: Date.now() });
    }
    setTimeout(loop, 500);
  }
  loop();
}

function stopSync() {}

connect();
