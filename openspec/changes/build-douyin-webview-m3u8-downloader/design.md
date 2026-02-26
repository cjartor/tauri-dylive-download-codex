## Context

The planned tool is a Tauri v2 desktop app that loads a fixed Douyin review URL in WebView and provides operator controls outside the page DOM. In-page DOM can change frequently due to SPA routing and async rendering, so the top action button and sidebar must be app-owned UI to remain stable across page transitions.

## Goals / Non-Goals

**Goals:**
- Embed `https://anchor.douyin.com/anchor/review` in a desktop app.
- Provide a persistent top button outside WebView to trigger discovery and open the sidebar.
- Discover m3u8 sources from page activity and present them in app sidebar UI.
- Support local download of selected streams with filename defaulting to `.basic-name`.
- Provide resilient behavior when page route/DOM changes.

**Non-Goals:**
- Modifying target page visual layout by injecting persistent UI elements into the page itself.
- Circumventing platform access controls or authentication systems.
- Building advanced download features (batch scheduling, resumable segmented recovery) in first iteration.

## Decisions

### Decision: Keep controls outside WebView
Top button and sidebar are rendered by app frontend, not injected into page DOM.

Alternative considered:
- Inject top button into target page. Rejected due to unstable DOM and route changes causing frequent button loss.

### Decision: Use WebView script bridge for data collection only
Inject lightweight collector script into page context to observe candidate m3u8 URLs (request interception + media inspection) and emit them to app via IPC/event bridge.

Alternative considered:
- Pure Rust network interception at transport layer. Rejected for higher complexity and lower portability in first version.

### Decision: Use service boundary for download execution
Frontend triggers download command with selected URL and computed file name; Rust side owns filesystem writes and download pipeline.

Alternative considered:
- Frontend-only download. Rejected because local file writing and robust long-running tasks are safer in Rust backend.

### Decision: Filename precedence from `.basic-name`
Collector attempts DOM query for `.basic-name`; if unavailable, fallback to timestamp-based safe filename.

Alternative considered:
- URL-derived filenames only. Rejected because they are often opaque and poor for operator workflows.

## Risks / Trade-offs

- [Risk] m3u8 URLs may be short-lived or tokenized.  
  Mitigation: show clear failure reason and support quick re-discovery.
- [Risk] Page internal changes may reduce extraction reliability.  
  Mitigation: combine multiple discovery strategies (network observer + media element scan + manual refresh action).
- [Risk] Download compatibility varies by stream format/encryption.  
  Mitigation: define supported stream types for v1 and fail fast with actionable errors.
- [Risk] Legal/compliance concerns for downloading protected content.  
  Mitigation: keep usage within authorized scope and present explicit operator responsibility note.

## Migration Plan

1. Scaffold Tauri app structure with split layout (top bar, webview area, right sidebar).
2. Add collector bridge and source model.
3. Add discovery action flow and sidebar rendering.
4. Add backend download command and progress events.
5. Validate with route changes on target page and local save workflow.

Rollback strategy:
- Disable collector/download modules behind a feature flag or revert the change if stability is insufficient.

## Open Questions

- Should v1 require external `ffmpeg` availability or use a pure Rust downloader path only?
- Do we need to persist historical discovered sources between app sessions?
- What minimum progress detail is required for UX (percentage vs phase-based status)?
