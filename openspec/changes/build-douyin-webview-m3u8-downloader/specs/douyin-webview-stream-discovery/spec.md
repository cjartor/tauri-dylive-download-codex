## ADDED Requirements

### Requirement: App SHALL embed fixed Douyin review page in WebView
The system SHALL load `https://anchor.douyin.com/anchor/review` as the primary content surface in the desktop application.

#### Scenario: Launch app to review page
- **WHEN** the user opens the desktop app
- **THEN** the WebView loads the configured Douyin review URL

### Requirement: Top discovery button SHALL be outside WebView
The system SHALL render the discovery trigger button in app-owned top UI outside the embedded page DOM so that route changes inside WebView do not remove the button.

#### Scenario: Button remains available after in-page route changes
- **WHEN** the user navigates within the loaded SPA page and the page DOM re-renders
- **THEN** the top discovery button remains visible and clickable without reinjection into page DOM

### Requirement: System SHALL discover m3u8 sources and show them in sidebar
The system SHALL collect candidate m3u8 stream URLs from page activity and display them in an app sidebar list.

#### Scenario: Discover sources after user trigger
- **WHEN** the user clicks the top discovery button and page activity includes m3u8 requests
- **THEN** the sidebar lists discovered m3u8 URLs for user selection

### Requirement: System SHALL capture display name from `.basic-name`
The system SHALL query `.basic-name` in the page DOM and attach its text as the default display name for discovered stream entries.

#### Scenario: Associate source with page-derived name
- **WHEN** an m3u8 source is discovered and `.basic-name` exists
- **THEN** the sidebar entry shows the extracted name as the default video title
