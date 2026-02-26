const state = {
  sidebarOpen: true,
  sources: new Map(),
};

const listEl = document.getElementById('source-list');
const discoverBtn = document.getElementById('discover-btn');
const clearBtn = document.getElementById('clear-btn');
const toggleSidebarBtn = document.getElementById('toggle-sidebar');
const sidebarEl = document.getElementById('sidebar');
const openReviewBtn = document.getElementById('open-review-btn');

const tauri = window.__TAURI__;

if (!tauri?.core?.invoke || !tauri?.event?.listen) {
  console.warn('Tauri global API not available. Ensure withGlobalTauri is enabled.');
}

function upsertSource(entry) {
  const url = (entry.url || '').trim();
  if (!url || !url.includes('.m3u8')) return;

  const existing = state.sources.get(url) || {
    url,
    title: 'Untitled Video',
    status: 'idle',
    progress: 0,
    message: '',
  };

  state.sources.set(url, {
    ...existing,
    title: sanitizeTitle(entry.title) || existing.title,
  });
  render();
}

function sanitizeTitle(raw) {
  return (raw || '').replace(/\s+/g, ' ').trim();
}

function fallbackTitle() {
  return `video_${Date.now()}`;
}

function render() {
  const entries = [...state.sources.values()];
  listEl.innerHTML = '';
  for (const item of entries) {
    const li = document.createElement('li');
    li.className = 'item';
    li.innerHTML = `
      <h4>${escapeHtml(item.title || fallbackTitle())}</h4>
      <div class="url">${escapeHtml(item.url)}</div>
      <div class="row">
        <button data-url="${encodeURIComponent(item.url)}">Download</button>
        <span class="status">${escapeHtml(statusLabel(item))}</span>
      </div>
    `;
    listEl.appendChild(li);
  }
}

function statusLabel(item) {
  if (item.status === 'in_progress') return `${item.progress || 0}% ${item.message || ''}`.trim();
  if (item.status === 'success') return `Done: ${item.outputPath || ''}`;
  if (item.status === 'failed') return `Failed: ${item.message || ''}`;
  if (item.status === 'started') return 'Starting...';
  return 'Idle';
}

function escapeHtml(input) {
  return input
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#039;');
}

discoverBtn.addEventListener('click', async () => {
  try {
    await tauri.core.invoke('discover_streams');
  } catch (err) {
    console.error('Discovery failed:', err);
  }
});

openReviewBtn.addEventListener('click', async () => {
  try {
    await tauri.core.invoke('open_review_window');
  } catch (err) {
    console.error('Open review window failed:', err);
  }
});

clearBtn.addEventListener('click', () => {
  state.sources.clear();
  render();
});

toggleSidebarBtn.addEventListener('click', () => {
  state.sidebarOpen = !state.sidebarOpen;
  sidebarEl.style.display = state.sidebarOpen ? 'flex' : 'none';
});

listEl.addEventListener('click', async (event) => {
  const btn = event.target.closest('button[data-url]');
  if (!btn) return;

  const url = decodeURIComponent(btn.dataset.url);
  const current = state.sources.get(url);
  if (!current) return;

  current.status = 'started';
  current.progress = 0;
  current.message = '';
  render();

  try {
    await tauri.core.invoke('start_download', {
      req: {
        m3u8Url: url,
        fileName: current.title || fallbackTitle(),
      },
    });
  } catch (err) {
    current.status = 'failed';
    current.message = String(err);
    render();
  }
});

if (tauri?.event?.listen) {
  tauri.event.listen('stream-discovered', (evt) => {
    upsertSource(evt.payload || {});
  });

  tauri.event.listen('download-status', (evt) => {
    const payload = evt.payload || {};
    const item = state.sources.get(payload.url);
    if (!item) return;

    item.status = payload.status || item.status;
    item.progress = payload.progress ?? item.progress;
    item.message = payload.message || item.message;
    item.outputPath = payload.outputPath || item.outputPath;
    render();
  });
}

render();
