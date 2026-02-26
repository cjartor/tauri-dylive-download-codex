## Why

We need a desktop workflow to review Douyin anchor content and quickly capture downloadable video streams without relying on fragile in-page UI injection. Building this now enables stable operator tooling around a known review page and reduces manual network inspection work.

## What Changes

- Build a Tauri v2 desktop app shell that embeds `https://anchor.douyin.com/anchor/review` in a WebView.
- Add an app-level top button outside the WebView to trigger stream discovery and control sidebar visibility.
- Add an app-level sidebar that lists discovered m3u8 sources for the current page context.
- Extract video display name from DOM element `.basic-name` and use it as the default download name.
- Implement local video download from selected m3u8 source with progress/state feedback.
- Preserve functionality across page route/DOM changes by keeping controls outside the WebView and running resilient source discovery.

## Capabilities

### New Capabilities
- `douyin-webview-stream-discovery`: Discover and surface m3u8 stream URLs from the embedded Douyin review page into app-managed UI.
- `douyin-m3u8-local-download`: Download selected m3u8 streams to local files using names derived from `.basic-name`.

### Modified Capabilities
- None.

## Impact

- Affected code: new Tauri app scaffolding (Rust + frontend), WebView bridge/injection scripts, sidebar and top-bar UI modules, download service.
- External systems: Douyin anchor review web page loaded inside WebView.
- Dependencies: likely m3u8 download pipeline dependency (native tool or Rust crate), plus Tauri shell/IPC integration.
- Risks: page behavior or request patterns may change; authenticated stream URLs may expire and require robust error handling.
