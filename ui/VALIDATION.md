# Validation Checklist

## Manual E2E Flow

1. Launch app with `npm run tauri:dev`.
2. Confirm top controls are visible before/after target page internal navigation.
3. Click `Discover Streams`.
4. If cross-origin auto capture is blocked, generate m3u8 requests in page and re-trigger discovery.
5. Confirm sidebar shows deduplicated m3u8 entries and title from `.basic-name` when available.
6. Click `Download` for one entry.
7. Verify download lifecycle in sidebar: started -> in_progress -> success or failed.
8. Confirm output file exists in current working directory with sanitized filename.

## Failure Cases

- Invalid/expired m3u8 URL should show failed status with actionable message.
- Missing `.basic-name` should fallback to timestamp-based filename.

## Current Status

- Code path implemented.
- Runtime validation pending environment setup (`npm install` + Tauri toolchain).
