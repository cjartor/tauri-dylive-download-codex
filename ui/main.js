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
  console.warn('未检测到 Tauri 全局 API，请确认 withGlobalTauri 已启用。');
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
    const displayTitle = item.title || '（缺少项目标题）';
    const actions = renderActions(item);
    const li = document.createElement('li');
    li.className = 'item';
    li.innerHTML = `
      <h4>${escapeHtml(displayTitle)}</h4>
      <div class="url">${escapeHtml(item.url)}</div>
      <div class="row">
        ${actions}
        <span class="status">${escapeHtml(statusLabel(item))}</span>
      </div>
    `;
    listEl.appendChild(li);
  }
}

function renderActions(item) {
  const u = encodeURIComponent(item.url);
  if (item.status === 'started' || item.status === 'in_progress') {
    return `<button data-action="pause" data-url="${u}">暂停</button>`;
  }
  if (item.status === 'paused') {
    return `<button data-action="resume" data-url="${u}">继续</button>`;
  }
  if (item.status === 'success' && item.outputPath) {
    return `<button class="text-btn" data-action="open-folder" data-path="${encodeURIComponent(item.outputPath)}">打开文件夹</button>`;
  }
  return `<button data-action="download" data-url="${u}">下载</button>`;
}

function statusLabel(item) {
  if (item.status === 'in_progress') return `${item.progress || 0}% ${item.message || ''}`.trim();
  if (item.status === 'paused') return `已暂停 ${item.progress || 0}%`;
  if (item.status === 'success') return `完成：${item.outputPath || ''}`;
  if (item.status === 'failed') return `失败：${item.message || ''}`;
  if (item.status === 'started') return '开始中...';
  return '空闲';
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
    console.error('发现视频源失败:', err);
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
  const btn = event.target.closest('button[data-action]');
  if (!btn) return;

  const action = btn.dataset.action;
  if (action === 'open-folder') {
    const outputPath = decodeURIComponent(btn.dataset.path || '');
    if (!outputPath) return;
    try {
      await tauri.core.invoke('open_download_folder', { path: outputPath });
    } catch (err) {
      console.error('打开文件夹失败:', err);
    }
    return;
  }

  const url = decodeURIComponent(btn.dataset.url || '');
  const current = state.sources.get(url);
  if (!current) return;

  if (action === 'pause') {
    try {
      await tauri.core.invoke('pause_download', { url });
      current.status = 'paused';
      current.message = '已手动暂停';
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
      current.message = '继续中...';
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
    const projectTitle = sanitizeTitle(current.title);
    if (!projectTitle) {
      current.status = 'failed';
      current.message = '缺少项目标题，请先在视频已加载页面点击“发现视频源”。';
      render();
      return;
    }

    const selectedPath = await tauri.core.invoke('pick_save_path', {
      fileName: projectTitle,
    });

    if (!selectedPath) {
      current.status = 'idle';
      current.message = '已取消';
      render();
      return;
    }

    await tauri.core.invoke('start_download', {
      req: {
        m3u8Url: url,
        fileName: projectTitle,
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
  const docHeight = document.documentElement?.clientHeight || window.innerHeight;
  const width = Math.max(320, window.innerWidth - sidebarWidth);
  const height = Math.max(220, docHeight - topbarHeight);

  tauri.core.invoke('layout_review_webview', {
    x: 0,
    y: topbarHeight,
    width,
    height,
  }).catch((err) => {
    console.error('子 WebView 布局失败:', err);
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
