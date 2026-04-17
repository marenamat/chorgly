// app.js — glue between the WASM module and the DOM
// Requires chorgly_frontend.js (wasm-bindgen output) in the same directory.

import init, { AppState, decode_server_msg, encode_client_msg } from './pkg/chorgly_frontend.js';

const WS_URL = window.location.protocol === 'https:'
  ? `wss://${window.location.hostname}:8765/`
  : `ws://${window.location.hostname}:8765/`;

const TOKEN_KEY = 'chorgly_token';

let ws = null;
let state = null; // AppState (WASM)

// ---- bootstrap ----

async function main() {
  await init(); // initialise WASM module
  state = new AppState();

  const saved = localStorage.getItem(TOKEN_KEY);
  if (saved) {
    showApp();
    connect(saved);
  } else {
    showAuth();
  }
}

// ---- screen helpers ----

function showAuth() {
  document.getElementById('auth-screen').hidden = false;
  document.getElementById('app-screen').hidden = true;
}

function showApp() {
  document.getElementById('auth-screen').hidden = true;
  document.getElementById('app-screen').hidden = false;
}

// ---- WebSocket ----

function connect(token) {
  ws = new WebSocket(WS_URL);
  ws.binaryType = 'arraybuffer';

  ws.addEventListener('open', () => {
    send({ Auth: { token } });
  });

  ws.addEventListener('message', (ev) => {
    const bytes = new Uint8Array(ev.data);
    let msg;
    try {
      msg = decode_server_msg(bytes);
    } catch (e) {
      console.error('failed to decode server message', e);
      return;
    }
    handleServerMsg(msg, token);
  });

  ws.addEventListener('close', () => {
    // Reconnect after 3 s.
    setTimeout(() => {
      const t = localStorage.getItem(TOKEN_KEY);
      if (t) connect(t);
    }, 3000);
  });

  ws.addEventListener('error', (e) => {
    console.error('WebSocket error', e);
  });
}

function send(msg) {
  if (!ws || ws.readyState !== WebSocket.OPEN) return;
  try {
    const bytes = encode_client_msg(JSON.stringify(msg));
    ws.send(bytes);
  } catch (e) {
    console.error('failed to encode message', e);
  }
}

// ---- server message handler ----

function handleServerMsg(msg, token) {
  const event = state.apply(msg);

  if (event === 'auth_ok') {
    localStorage.setItem(TOKEN_KEY, token);
    document.getElementById('user-name').textContent = msg.AuthOk.user.name;
    showApp();
    send('ListChores');
    return;
  }

  if (event === 'auth_fail') {
    localStorage.removeItem(TOKEN_KEY);
    document.getElementById('auth-error').textContent =
      msg.AuthFail?.reason ?? 'Login failed.';
    document.getElementById('auth-error').hidden = false;
    showAuth();
    return;
  }

  if (event === 'error') {
    console.warn('server error:', msg.Error?.reason);
    return;
  }

  // snapshot / chore_changed / chore_deleted → re-render
  renderChores();
}

// ---- rendering ----

function renderChores() {
  const list = document.getElementById('chore-list');
  const empty = document.getElementById('no-chores');

  let chores;
  try {
    chores = state.pending_chores_json();
  } catch (e) {
    console.error('pending_chores_json failed', e);
    return;
  }

  list.innerHTML = '';

  if (!chores || chores.length === 0) {
    empty.hidden = false;
    return;
  }
  empty.hidden = true;

  const now = Date.now();

  for (const c of chores) {
    const li = document.createElement('li');
    li.className = 'chore-item';
    if (c.depends_on && c.depends_on.length > 0) li.classList.add('blocked');

    const dueMs = c.next_due ? new Date(c.next_due).getTime() : null;
    if (dueMs && dueMs < now) li.classList.add('overdue');

    const title = document.createElement('span');
    title.className = 'chore-title';
    title.textContent = c.title;

    const due = document.createElement('span');
    due.className = 'chore-due';
    due.textContent = dueMs ? formatDue(dueMs, now) : '';

    const btn = document.createElement('button');
    btn.className = 'chore-done-btn';
    btn.textContent = 'Done';
    btn.disabled = li.classList.contains('blocked');
    btn.addEventListener('click', () => {
      send({ CompleteChore: { chore_id: c.id } });
    });

    li.append(title, due, btn);
    list.append(li);
  }
}

function formatDue(dueMs, nowMs) {
  const diff = dueMs - nowMs;
  const abs = Math.abs(diff);
  const mins  = Math.floor(abs / 60000);
  const hours = Math.floor(abs / 3600000);
  const days  = Math.floor(abs / 86400000);

  const label = days > 0 ? `${days}d` : hours > 0 ? `${hours}h` : `${mins}m`;
  return diff < 0 ? `overdue ${label}` : `due in ${label}`;
}

// ---- auth form ----

document.getElementById('auth-form').addEventListener('submit', (e) => {
  e.preventDefault();
  const token = document.getElementById('token-input').value.trim();
  if (!token) return;
  connect(token);
});

// ---- add-chore buttons ----

const dialog = document.getElementById('add-chore-dialog');
let addingPersonal = false;

document.getElementById('btn-add-common').addEventListener('click', () => {
  addingPersonal = false;
  document.getElementById('add-chore-title').textContent = 'Add common chore';
  dialog.showModal();
});

document.getElementById('btn-add-personal').addEventListener('click', () => {
  addingPersonal = true;
  document.getElementById('add-chore-title').textContent = 'Add my chore';
  dialog.showModal();
});

document.getElementById('btn-cancel-dialog').addEventListener('click', () => {
  dialog.close();
});

// Show/hide extra fields based on chore kind.
document.getElementById('chore-kind').addEventListener('change', (e) => {
  document.getElementById('field-delay').hidden    = e.target.value !== 'RecurringAfterCompletion';
  document.getElementById('field-schedule').hidden = e.target.value !== 'RecurringScheduled';
  document.getElementById('field-deadline').hidden = e.target.value !== 'WithDeadline';
});

document.getElementById('add-chore-form').addEventListener('submit', (e) => {
  e.preventDefault();

  const title = document.getElementById('chore-name').value.trim();
  const kindKey = document.getElementById('chore-kind').value;

  let kind;
  if (kindKey === 'OneTime') {
    kind = 'OneTime';
  } else if (kindKey === 'RecurringAfterCompletion') {
    const hours = parseInt(document.getElementById('chore-delay').value, 10) || 168;
    kind = { RecurringAfterCompletion: { delay_secs: hours * 3600 } };
  } else if (kindKey === 'RecurringScheduled') {
    const schedule = document.getElementById('chore-schedule').value.trim();
    kind = { RecurringScheduled: { schedule } };
  } else {
    const deadline = document.getElementById('chore-deadline').value;
    kind = { WithDeadline: { deadline } };
  }

  // assigned_to: null → common; [current user id] → personal
  // We don't have a clean way to get the user id here without exposing it from WASM.
  // For the prototype, personal chores are marked by passing an empty-but-non-null list;
  // the server will fill in the authenticated user's id (TODO: clarify in design).
  const assigned_to = addingPersonal ? [] : null;

  send({ AddChore: { title, kind, assigned_to, depends_on: [] } });
  dialog.close();
});

// ---- init ----
main().catch(console.error);
