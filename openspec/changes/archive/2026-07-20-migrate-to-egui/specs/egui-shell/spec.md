## ADDED Requirements

### Requirement: Single-process egui desktop shell

The system SHALL run as a single-process native desktop application using egui/eframe, with no embedded WebView or separate frontend runtime process required for the main UI.

#### Scenario: Launch main window

- **WHEN** the user starts the application
- **THEN** a native window opens presenting the logcat UI without spawning a WebView content process for that UI

### Requirement: Main window hosts core logcat surfaces

The system SHALL present within the main window: device/streaming toolbar controls, filter controls, a scrollable log list, and a status bar showing buffer usage.

#### Scenario: Initial layout visible

- **WHEN** the main window is shown after launch
- **THEN** toolbar, filter area, log list region, and status bar are all visible and usable

### Requirement: Native file dialogs for export paths

The system SHALL use native OS file dialogs when the user chooses an export destination.

#### Scenario: Export pick path

- **WHEN** the user initiates an export that requires choosing a file path
- **THEN** a native save dialog is shown and the chosen path is used for the export
