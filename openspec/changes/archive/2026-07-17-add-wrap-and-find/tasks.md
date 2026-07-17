## 1. Soft-Wrap

- [x] 1.1 Add `useSoftWrap` hook with localStorage persistence (`ohmylogcat.softWrap`, default `false`)
- [x] 1.2 Add Soft-Wrap toggle button to `Toolbar.tsx` (label/tooltip aligned with AS "Use Soft Wraps")
- [x] 1.3 Update `LogList.tsx`: remove `truncate`; apply `whitespace-nowrap overflow-x-auto` when wrap off
- [x] 1.4 Update `LogList.tsx`: apply `whitespace-pre-wrap break-all` when wrap on; verify Virtuoso variable-height scrolling

## 2. Find in Log

- [x] 2.1 Create `useFindInLog` hook: scan visible entries case-insensitively, return matches `{ lineIndex, start, end }[]` and navigation helpers
- [x] 2.2 Create `FindBar.tsx`: search input, match counter (e.g. 2/15), prev/next buttons, close button
- [x] 2.3 Wire `FindBar` and find state in `App.tsx`; pass find props into `LogList`
- [x] 2.4 Implement highlight rendering in `LogList.tsx`: `<mark>` for all matches, stronger highlight for current match
- [x] 2.5 Scroll to current match via `virtuosoRef.scrollToIndex({ align: 'center' })` on navigation and query change

## 3. Keyboard Shortcuts

- [x] 3.1 Add `useKeyboardShortcuts` (or inline in App): Cmd+F / Ctrl+F opens find bar and focuses input
- [x] 3.2 Bind Enter → next match, Shift+Enter → previous match, Esc → close find bar
- [x] 3.3 Call `preventDefault` on Cmd+F / Ctrl+F to suppress WebView default find

## 4. Integration & Verification

- [x] 4.1 Recompute matches when `entries` or filter results change; clamp current index when out of bounds
- [x] 4.2 Manual test: long lines with wrap off (horizontal scroll) and wrap on (line break)
- [x] 4.3 Manual test: find within filtered results preserves all visible lines; next/prev wraps at boundaries
- [x] 4.4 Manual test: Soft-Wrap preference survives app restart
