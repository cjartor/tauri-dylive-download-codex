## 1. App Shell and Layout

- [x] 1.1 Scaffold Tauri v2 app structure with a fixed WebView loading `https://anchor.douyin.com/anchor/review`.
- [x] 1.2 Implement app-owned top bar button outside WebView for discovery trigger.
- [x] 1.3 Implement app-owned right sidebar container for stream list and download actions.

## 2. Stream Discovery Bridge

- [x] 2.1 Add WebView-side collector script to discover candidate m3u8 URLs from page activity.
- [x] 2.2 Extract `.basic-name` from page DOM and attach it to discovered stream entries.
- [x] 2.3 Bridge discovered entries from WebView context to app UI state and render deduplicated list items.

## 3. Download Pipeline

- [x] 3.1 Implement backend command to start local download from selected m3u8 URL.
- [x] 3.2 Implement filename sanitization using `.basic-name` with fallback naming when missing.
- [x] 3.3 Add download status/progress events and bind sidebar status updates (started, in progress, success, failed).

## 4. Reliability and Validation

- [x] 4.1 Ensure discovery and listing remain usable across in-page route/DOM changes without losing top button.
- [x] 4.2 Add error handling for expired/invalid URLs and show actionable user feedback.
- [x] 4.3 Validate end-to-end flow: discover source, show name, download locally, and report final result.
