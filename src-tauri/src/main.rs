use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::webview::WebviewBuilder;
use tauri::{Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl};
use tauri_plugin_dialog::DialogExt;
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadRequest {
    m3u8_url: String,
    file_name: Option<String>,
    output_dir: Option<String>,
    output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadEvent {
    url: String,
    status: String,
    progress: u8,
    message: Option<String>,
    output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscoveredSource {
    url: String,
    title: Option<String>,
}

#[derive(Default)]
struct DownloadControl {
    paused: Mutex<HashMap<String, bool>>,
}

impl DownloadControl {
    fn register(&self, url: &str) {
        if let Ok(mut map) = self.paused.lock() {
            map.insert(url.to_string(), false);
        }
    }

    fn remove(&self, url: &str) {
        if let Ok(mut map) = self.paused.lock() {
            map.remove(url);
        }
    }

    fn set_paused(&self, url: &str, paused: bool) -> bool {
        if let Ok(mut map) = self.paused.lock() {
            if let Some(v) = map.get_mut(url) {
                *v = paused;
                return true;
            }
        }
        false
    }

    fn is_paused(&self, url: &str) -> bool {
        if let Ok(map) = self.paused.lock() {
            return map.get(url).copied().unwrap_or(false);
        }
        false
    }
}

const REVIEW_WEBVIEW_LABEL: &str = "review-webview";
const REVIEW_URL: &str = "https://anchor.douyin.com/anchor/review";

const REVIEW_COLLECTOR_INIT: &str = r#"
(() => {
  if (window.__TAURI_M3U8_COLLECTOR_INSTALLED__) return;
  window.__TAURI_M3U8_COLLECTOR_INSTALLED__ = true;

  const seen = new Set();
  const title = () => (document.querySelector('.basic-name')?.textContent || '').trim();
  const emit = (url) => {
    if (!url || !url.includes('.m3u8') || seen.has(url)) return;
    seen.add(url);
    if (window.__TAURI__?.core?.invoke) {
      window.__TAURI__.core.invoke('upsert_discovered_source', {
        entry: { url, title: title() }
      }).catch(() => {});
    }
  };

  window.__TAURI_M3U8_DISCOVER__ = () => {
    for (const e of performance.getEntriesByType('resource')) emit(e.name || '');
    for (const node of document.querySelectorAll('video, source')) {
      emit(node.currentSrc || node.src || '');
    }
  };

  const oldFetch = window.fetch;
  window.fetch = async (...args) => {
    const input = args[0];
    const url = typeof input === 'string' ? input : input?.url;
    emit(url || '');
    return oldFetch(...args);
  };

  const oldOpen = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function(method, url, ...rest) {
    emit(String(url || ''));
    return oldOpen.call(this, method, url, ...rest);
  };

  setInterval(() => {
    if (window.__TAURI_M3U8_DISCOVER__) window.__TAURI_M3U8_DISCOVER__();
  }, 2000);
})();
"#;

#[tauri::command]
fn layout_review_webview(
    app: tauri::AppHandle,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    ensure_review_webview(&app)?;
    let review = app
        .get_webview(REVIEW_WEBVIEW_LABEL)
        .ok_or_else(|| "review webview not found".to_string())?;

    let safe_x = x.max(0.0);
    let safe_y = y.max(0.0);
    let safe_w = width.max(320.0);
    let safe_h = height.max(200.0);

    review
        .set_position(LogicalPosition::new(safe_x, safe_y))
        .map_err(|e| format!("set review webview position failed: {e}"))?;
    review
        .set_size(LogicalSize::new(safe_w, safe_h))
        .map_err(|e| format!("set review webview size failed: {e}"))?;

    Ok(())
}

#[tauri::command]
fn discover_streams(app: tauri::AppHandle) -> Result<(), String> {
    ensure_review_webview(&app)?;
    let review = app
        .get_webview(REVIEW_WEBVIEW_LABEL)
        .ok_or_else(|| "review webview not found".to_string())?;

    review
        .eval("window.__TAURI_M3U8_DISCOVER__ && window.__TAURI_M3U8_DISCOVER__();")
        .map_err(|e| format!("discover eval failed: {e}"))
}

#[tauri::command]
async fn pick_save_path(app: tauri::AppHandle, file_name: Option<String>) -> Result<Option<String>, String> {
    let suggested = sanitize_name(file_name.as_deref().unwrap_or("video"));
    let path = app
        .dialog()
        .file()
        .set_file_name(format!("{suggested}.mp4"))
        .blocking_save_file();

    if let Some(file_path) = path {
        let pb = file_path
            .into_path()
            .map_err(|e| format!("invalid selected path: {e}"))?;
        return Ok(Some(pb.to_string_lossy().to_string()));
    }

    Ok(None)
}

#[tauri::command]
fn pause_download(control: tauri::State<'_, DownloadControl>, url: String) -> Result<(), String> {
    if control.set_paused(&url, true) {
        Ok(())
    } else {
        Err("download task not found for pause".into())
    }
}

#[tauri::command]
fn resume_download(control: tauri::State<'_, DownloadControl>, url: String) -> Result<(), String> {
    if control.set_paused(&url, false) {
        Ok(())
    } else {
        Err("download task not found for resume".into())
    }
}

#[tauri::command]
async fn start_download(
    app: tauri::AppHandle,
    control: tauri::State<'_, DownloadControl>,
    req: DownloadRequest,
) -> Result<String, String> {
    control.register(&req.m3u8_url);
    let result: Result<String, String> = async {
        emit_download(
            &app,
            DownloadEvent {
                url: req.m3u8_url.clone(),
                status: "started".into(),
                progress: 0,
                message: None,
                output_path: None,
            },
        );

        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(16)
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .build()
            .map_err(|e| format!("create http client failed: {e}"))?;

        let safe_name = sanitize_name(
            req.file_name
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("video"),
        );
        let output_file = if let Some(selected) = req.output_path.clone() {
            let mut p = PathBuf::from(selected);
            p.set_extension("mp4");
            if let Some(parent) = p.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("create output dir failed: {e}"))?;
                }
            }
            p
        } else {
            let out_dir = req
                .output_dir
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            if !out_dir.exists() {
                fs::create_dir_all(&out_dir).map_err(|e| format!("create output dir failed: {e}"))?;
            }
            unique_path(&out_dir, &safe_name, "mp4")
        };

        let first_playlist = fetch_text_with_client(&client, &req.m3u8_url).await?;
        let media_playlist_url = choose_media_playlist_url(&req.m3u8_url, &first_playlist)?;
        let media_playlist = fetch_text_with_client(&client, &media_playlist_url).await?;
        let segments = parse_segments(&media_playlist_url, &media_playlist)?;

        if segments.is_empty() {
            return Err("no downloadable ts segments found in m3u8".into());
        }

        let mut file = File::create(&output_file).map_err(|e| format!("create file failed: {e}"))?;
        let total = segments.len();
        let max_inflight = 12usize;
        let mut next_to_schedule = 0usize;
        let mut next_to_write = 0usize;
        let mut paused_emitted = false;
        let mut inflight = JoinSet::<Result<(usize, Vec<u8>), String>>::new();
        let mut ready: HashMap<usize, Vec<u8>> = HashMap::new();

        while next_to_write < total {
            while !control.is_paused(&req.m3u8_url)
                && next_to_schedule < total
                && inflight.len() < max_inflight
            {
                let idx = next_to_schedule;
                let seg_url = segments[idx].clone();
                let c = client.clone();
                inflight.spawn(async move {
                    let resp = c
                        .get(&seg_url)
                        .send()
                        .await
                        .map_err(|e| format!("segment request failed: {e}"))?;
                    let bytes = resp
                        .bytes()
                        .await
                        .map_err(|e| format!("segment read failed: {e}"))?;
                    Ok((idx, bytes.to_vec()))
                });
                next_to_schedule += 1;
            }

            if control.is_paused(&req.m3u8_url) {
                if !paused_emitted {
                    emit_download(
                        &app,
                        DownloadEvent {
                            url: req.m3u8_url.clone(),
                            status: "paused".into(),
                            progress: (((next_to_write) as f32 / total as f32) * 100.0).round() as u8,
                            message: Some("download paused".into()),
                            output_path: None,
                        },
                    );
                    paused_emitted = true;
                }
                sleep(Duration::from_millis(300)).await;
                continue;
            }

            if paused_emitted {
                emit_download(
                    &app,
                    DownloadEvent {
                        url: req.m3u8_url.clone(),
                        status: "in_progress".into(),
                        progress: (((next_to_write) as f32 / total as f32) * 100.0).round() as u8,
                        message: Some("download resumed".into()),
                        output_path: None,
                    },
                );
                paused_emitted = false;
            }

            if let Some(joined) = inflight.join_next().await {
                let result = joined.map_err(|e| format!("download worker join failed: {e}"))?;
                let (idx, bytes) = result?;
                ready.insert(idx, bytes);
            } else if next_to_schedule >= total {
                return Err("download interrupted unexpectedly".into());
            }

            while let Some(bytes) = ready.remove(&next_to_write) {
                file.write_all(&bytes)
                    .map_err(|e| format!("write segment failed: {e}"))?;

                next_to_write += 1;
                let pct = ((next_to_write as f32 / total as f32) * 100.0).round() as u8;
                emit_download(
                    &app,
                    DownloadEvent {
                        url: req.m3u8_url.clone(),
                        status: "in_progress".into(),
                        progress: pct,
                        message: Some(format!("downloaded {}/{} segments", next_to_write, total)),
                        output_path: None,
                    },
                );
            }
        }

        let out = output_file.to_string_lossy().to_string();
        emit_download(
            &app,
            DownloadEvent {
                url: req.m3u8_url.clone(),
                status: "success".into(),
                progress: 100,
                message: Some("download completed".into()),
                output_path: Some(out.clone()),
            },
        );

        Ok(out)
    }
    .await;

    control.remove(&req.m3u8_url);
    result
}

#[tauri::command]
async fn upsert_discovered_source(
    app: tauri::AppHandle,
    entry: DiscoveredSource,
) -> Result<(), String> {
    app.emit("stream-discovered", entry)
        .map_err(|e| format!("emit failed: {e}"))
}

fn ensure_review_webview(app: &tauri::AppHandle) -> Result<(), String> {
    if app.get_webview(REVIEW_WEBVIEW_LABEL).is_some() {
        return Ok(());
    }

    let main_window = app
        .get_window("main")
        .ok_or_else(|| "main window not found".to_string())?;

    let url = Url::parse(REVIEW_URL).map_err(|e| format!("invalid review url: {e}"))?;
    let builder = WebviewBuilder::new(REVIEW_WEBVIEW_LABEL, WebviewUrl::External(url))
        .initialization_script_for_all_frames(REVIEW_COLLECTOR_INIT);

    main_window
        .add_child(
            builder,
            LogicalPosition::new(0.0, 120.0),
            LogicalSize::new(1024.0, 640.0),
        )
        .map_err(|e| format!("create child review webview failed: {e}"))?;

    Ok(())
}

fn emit_download(app: &tauri::AppHandle, evt: DownloadEvent) {
    let _ = app.emit("download-status", evt);
}

fn sanitize_name(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == ' ' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let out = out.trim().replace(' ', "_");
    if out.is_empty() {
        format!("video_{}", chrono_like_timestamp())
    } else {
        out
    }
}

fn unique_path(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let mut idx = 0usize;
    loop {
        let candidate = if idx == 0 {
            dir.join(format!("{stem}.{ext}"))
        } else {
            dir.join(format!("{stem}_{idx}.{ext}"))
        };
        if !candidate.exists() {
            return candidate;
        }
        idx += 1;
    }
}

fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    ts.to_string()
}

async fn fetch_text_with_client(client: &reqwest::Client, url: &str) -> Result<String, String> {
    client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?
        .text()
        .await
        .map_err(|e| format!("read text failed: {e}"))
}

fn choose_media_playlist_url(base: &str, text: &str) -> Result<String, String> {
    let mut variant = None;
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.ends_with(".m3u8") || line.contains(".m3u8?") {
            variant = Some(line.to_string());
            break;
        }
    }

    if let Some(v) = variant {
        return resolve_url(base, &v);
    }

    if text.contains("#EXTINF") {
        return Ok(base.to_string());
    }

    Err("invalid m3u8 content".into())
}

fn parse_segments(base: &str, text: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.ends_with(".ts") || line.contains(".ts?") || line.contains("/segment") {
            out.push(resolve_url(base, line)?);
        }
    }
    Ok(out)
}

fn resolve_url(base: &str, candidate: &str) -> Result<String, String> {
    if candidate.starts_with("http://") || candidate.starts_with("https://") {
        return Ok(candidate.to_string());
    }

    let base = Url::parse(base).map_err(|e| format!("bad base url: {e}"))?;
    base.join(candidate)
        .map(|u| u.to_string())
        .map_err(|e| format!("url join failed: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(DownloadControl::default())
        .setup(|app| {
            let handle = app.handle().clone();
            ensure_review_webview(&handle)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_download,
            upsert_discovered_source,
            discover_streams,
            layout_review_webview,
            pick_save_path,
            pause_download,
            resume_download
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
