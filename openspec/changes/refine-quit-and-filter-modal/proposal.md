## Why

Ctrl+C as quit conflicts with Windows users' copy muscle memory and with toolbar shortcuts like `[c]Clear`. Inline Tag/Message filter editing also forces special-case handling for every letter shortcut (including `q`), making a pseudo-global quit rule fragile. A clear layer model—top layer quits with `q`, overlay layers never quit—simplifies keyboard behavior and matches how other modals already work.

## What Changes

- Remove Ctrl+C as a quit shortcut (**BREAKING** for users who relied on it; `q` remains on the top layer).
- Quit with `q` only on the top layer (`no modal open` and find bar closed); overlay layers never trigger quit via `q`.
- On overlay layers with text input (filter modals, find, export path, settings text fields), `q` inserts the character; on non-text overlays (Devices, Export menu, etc.), `q` has no effect—only Esc returns to the top layer.
- Replace inline Tag/Message filter editing with modal overlay editors opened via click or `t`/`m`; filter changes apply in real time as the user types; Esc only closes the modal (independent of apply logic).
- Add `[q]Quit` to the toolbar shortcut hints.
- Level filter control and inline Find behavior remain unchanged.

## Capabilities

### New Capabilities

<!-- None -->

### Modified Capabilities

- `tui-shell`: Quit shortcut rules (top-layer-only `q`, remove Ctrl+C); toolbar quit hint; Tag/Message filter editing via modal with live apply; updated focus/layer model; Esc closes overlays without reverting live filter values.

## Impact

- `src/app.rs`: key routing, `ModalKind` / filter UI, toolbar labels, removal of `Focus::Tag` / `Focus::Message` inline editing paths.
- `README.md`: keyboard cheat sheet (quit row, filter editing flow).
- `openspec/specs/tui-shell/spec.md`: requirement updates for filter modals and quit behavior.
