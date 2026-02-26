## ADDED Requirements

### Requirement: System SHALL download selected m3u8 stream to local filesystem
The system SHALL allow users to start a local download for a selected m3u8 source from the sidebar.

#### Scenario: Start download from sidebar action
- **WHEN** the user clicks download on a listed m3u8 source
- **THEN** the system starts a local download task for that source

### Requirement: Download filename SHALL default to `.basic-name`
The system SHALL use the value extracted from `.basic-name` as default filename base for downloaded video artifacts.

#### Scenario: Save file using extracted title
- **WHEN** `.basic-name` text is available for the selected source
- **THEN** the saved file name uses that text (after filesystem-safe sanitization)

### Requirement: System SHALL provide fallback filename
The system SHALL generate a deterministic fallback filename when `.basic-name` is unavailable or empty.

#### Scenario: Missing title in page DOM
- **WHEN** the selected source has no usable `.basic-name` value
- **THEN** the system saves with a fallback name derived from timestamp or source identifier

### Requirement: System SHALL expose download status to UI
The system SHALL provide task status updates (started, in progress, success, failed) so the sidebar can reflect current state.

#### Scenario: Report progress lifecycle
- **WHEN** a download task is running
- **THEN** the sidebar updates status until completion or error
