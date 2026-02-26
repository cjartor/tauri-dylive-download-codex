const state = {
  sidebarOpen: true,
  sources: new Map(),
};

const listEl = document.getElementById('source-list');
const discoverBtn = document.getElementById('discover-btn');
const clearBtn = document.getElementById('clear-btn');
const toggleSidebarBtn = document.getElementById('toggle-sidebar');
const sidebarEl = document.getElementById('sidebar');

const tauri = window.__TAURI__;

if (!tauri?.core?.invoke || !tauri?.event?.listen) {
  console.warn('Tauri global API not available. Ensure withGlobalTauri is enabled.');
}

function upsertSource(entry) {
  const url = (entry.url || '').trim();
  if (!url || !url.includes('.m3u8')) return;

  const existing = state.sources.get(url) || {
    url,
    title: '',
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

function render() {
  const entries = [...state.sources.values()];
  listEl.innerHTML = '';
  for (const item of entries) {
    const displayTitle = item.title || '(missing .basic-name)';
    const li = document.createElement('li');
    li.className = 'item';
    li.innerHTML = `
      <h4>${escapeHtml(displayTitle)}</h4>
      <div class="url">${escapeHtml(item.url)}</div>
      <div class="row">
        <button data-action="download" data-url="${encodeURIComponent(item.url)}">Download</button>
        <button data-action="pause" data-url="${encodeURIComponent(item.url)}">Pause</button>
        <button data-action="resume" data-url="${encodeURIComponent(item.url)}">Resume</button>
        <span class="status">${escapeHtml(statusLabel(item))}</span>
      </div>
    `;
    listEl.appendChild(li);
  }
}

function statusLabel(item) {
  if (item.status === 'in_progress') return `${item.progress || 0}% ${item.message || ''}`.trim();
  if (item.status === 'paused') return `Paused ${item.progress || 0}%`;
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

clearBtn.addEventListener('click', () => {
  state.sources.clear();
  render();
});

toggleSidebarBtn.addEventListener('click', () => {
  state.sidebarOpen = !state.sidebarOpen;
  sidebarEl.style.display = state.sidebarOpen ? 'flex' : 'none';
  syncReviewWebviewLayout();
});

listEl.addEventListener('click', async (event) => {
  const btn = event.target.closest('button[data-action][data-url]');
  if (!btn) return;

  const action = btn.dataset.action;
  const url = decodeURIComponent(btn.dataset.url);
  const current = state.sources.get(url);
  if (!current) return;

  if (action === 'pause') {
    try {
      await tauri.core.invoke('pause_download', { url });
      current.status = 'paused';
      current.message = 'Paused by user';
      render();
    } catch (err) {
      current.status = 'failed';
      current.message = String(err);
      render();
    }
    return;
  }

  if (action === 'resume') {
    try {
      await tauri.core.invoke('resume_download', { url });
      current.status = 'in_progress';
      current.message = 'Resuming...';
      render();
    } catch (err) {
      current.status = 'failed';
      current.message = String(err);
      render();
    }
    return;
  }

  current.status = 'started';
  current.progress = 0;
  current.message = '';
  render();

  try {
    const baseName = sanitizeTitle(current.title);
    if (!baseName) {
      current.status = 'failed';
      current.message = 'Missing .basic-name; trigger discovery on a loaded video first.';
      render();
      return;
    }

    const selectedPath = await tauri.core.invoke('pick_save_path', {
      fileName: baseName,
    });

    if (!selectedPath) {
      current.status = 'idle';
      current.message = 'Cancelled';
      render();
      return;
    }

    await tauri.core.invoke('start_download', {
      req: {
        m3u8Url: url,
        fileName: baseName,
        outputPath: selectedPath,
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

let layoutFrame = null;
const TOP_GAP = 32;

function topbarOffset() {
  const topbar = document.querySelector('.topbar');
  const rect = topbar?.getBoundingClientRect?.();
  const bottom = rect?.bottom ?? 64;
  return Math.ceil(bottom + TOP_GAP);
}

function syncReviewWebviewLayout() {
  if (!tauri?.core?.invoke) return;
  const topbarHeight = topbarOffset();
  const sidebarWidth = state.sidebarOpen ? 420 : 0;
  const width = Math.max(320, window.innerWidth - sidebarWidth);
  const height = Math.max(220, window.innerHeight - topbarHeight);

  tauri.core.invoke('layout_review_webview', {
    x: 0,
    y: topbarHeight,
    width,
    height,
  }).catch((err) => {
    console.error('Layout review webview failed:', err);
  });
}

window.addEventListener('resize', () => {
  if (layoutFrame) cancelAnimationFrame(layoutFrame);
  layoutFrame = requestAnimationFrame(syncReviewWebviewLayout);
});

syncReviewWebviewLayout();
setTimeout(syncReviewWebviewLayout, 120);
setTimeout(syncReviewWebviewLayout, 400);

render();
