# Proposal: Log viewport text selection

## Problem

Mouse capture (`EnableMouseCapture`) intercepts drag events from the terminal emulator. The app only handles scroll and click, so users cannot drag-select log text or copy it. This blocks a core workflow for log inspection.

## Solution

Implement in-app text selection in the log viewport (Option B):

- Drag to select across formatted log lines (logical entry semantics in soft-wrap mode)
- Visual highlight distinct from find highlights
- Copy via Cmd+C (macOS) / Ctrl+C (Windows) when a selection exists
- I-beam cursor over log viewport; default cursor over toolbar, filters, and status bar

## Scope

- In scope: log viewport mouse drag selection, copy shortcut, contextual cursor
- Out of scope: optional disable-mouse-capture setting, stdin pipe mode

## Capabilities

- `tui-shell`: mouse cursor behavior, selection interaction rules
- `log-display`: selection rendering and copy semantics
