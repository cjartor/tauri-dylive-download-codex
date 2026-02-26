use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadRequest {
    m3u8_url: String,
    file_name: Option<String>,
    output_dir: Option<String>,
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

#[tauri::command]
async fn start_download(app: tauri::AppHandle, req: DownloadRequest) -> Result<String, String> {
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

    let out_dir = req
        .output_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    if !out_dir.exists() {
        fs::create_dir_all(&out_dir).map_err(|e| format!("create output dir failed: {e}"))?;
    }

    let safe_name = sanitize_name(
        req.file_name
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("video"),
    );

    let output_file = unique_path(&out_dir, &safe_name, "ts");

    let first_playlist = fetch_text(&req.m3u8_url).await?;
    let media_playlist_url = choose_media_playlist_url(&req.m3u8_url, &first_playlist)?;
    let media_playlist = fetch_text(&media_playlist_url).await?;
    let segments = parse_segments(&media_playlist_url, &media_playlist)?;

    if segments.is_empty() {
        return Err("no downloadable ts segments found in m3u8".into());
    }

    let mut file = File::create(&output_file).map_err(|e| format!("create file failed: {e}"))?;
    let total = segments.len();

    for (idx, seg_url) in segments.iter().enumerate() {
        let bytes = reqwest::get(seg_url)
            .await
            .map_err(|e| format!("segment request failed: {e}"))?
            .bytes()
            .await
            .map_err(|e| format!("segment read failed: {e}"))?;

        file.write_all(&bytes)
            .map_err(|e| format!("write segment failed: {e}"))?;

        let pct = (((idx + 1) as f32 / total as f32) * 100.0).round() as u8;
        emit_download(
            &app,
            DownloadEvent {
                url: req.m3u8_url.clone(),
                status: "in_progress".into(),
                progress: pct,
                message: Some(format!("downloaded {}/{} segments", idx + 1, total)),
                output_path: None,
            },
        );
    }

    let out = output_file.to_string_lossy().to_string();
    emit_download(
        &app,
        DownloadEvent {
            url: req.m3u8_url,
            status: "success".into(),
            progress: 100,
            message: Some("download completed".into()),
            output_path: Some(out.clone()),
        },
    );

    Ok(out)
}

#[tauri::command]
async fn upsert_discovered_source(
    app: tauri::AppHandle,
    entry: DiscoveredSource,
) -> Result<(), String> {
    app.emit("stream-discovered", entry)
        .map_err(|e| format!("emit failed: {e}"))
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

async fn fetch_text(url: &str) -> Result<String, String> {
    reqwest::get(url)
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
        .invoke_handler(tauri::generate_handler![start_download, upsert_discovered_source])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
