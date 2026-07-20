## REMOVED Requirements

### Requirement: Single-process egui desktop shell

**Reason**: Product direction moved to an extreme-lightweight TUI shell; egui/eframe (and its GPU baseline) are removed.

**Migration**: Use `tui-shell` — launch in a terminal; no separate egui window.

### Requirement: Main window hosts core logcat surfaces

**Reason**: Replaced by terminal layout requirements under `tui-shell`.

**Migration**: Toolbar, filters, log viewport, and status bar are hosted in the TUI main layout.

### Requirement: Native file dialogs for export paths

**Reason**: TUI uses in-terminal path entry instead of OS native save dialogs.

**Migration**: Export flow prompts for a path inside a TUI modal (see `log-export` and `tui-shell`).
