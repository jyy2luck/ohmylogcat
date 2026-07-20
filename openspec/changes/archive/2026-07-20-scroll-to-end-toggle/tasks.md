## 1. Scroll-to-end state hook

- [x] 1.1 Add `src/hooks/useScrollToEnd.ts` with localStorage key `ohmylogcat.scrollToEnd`, default `true`, and `setScrollToEndEnabled` that persists on change
- [x] 1.2 Replace `useState` in `App.tsx` with `useScrollToEnd`; wire toggle handler (OFF→ON: enable + scroll; ON→OFF: disable)
- [x] 1.3 Ensure scroll-up auto-disable in `LogList` writes through to persisted state via `setScrollToEndEnabled(false)`

## 2. Reliable tail-following in LogList

- [x] 2.1 Change Virtuoso `followOutput` to callback form that returns `"auto"` when tail-following is on and find is inactive, else `false`
- [x] 2.2 Add `useEffect` fallback: when `entries` changes and tail-following is on (and find inactive), call `scrollToIndex({ index: "LAST", align: "end" })`
- [x] 2.3 Use `index: "LAST"` in existing `scrollToEnd` imperative helper for consistency

## 3. Toolbar and integration

- [x] 3.1 Update Toolbar button `title`/accessibility text to describe tail-following toggle (on/off)
- [x] 3.2 Verify tail-following stays enabled across Clear and device switch without extra user action

## 4. Verification

- [x] 4.1 Manual test: tail-following ON + streaming logs stays pinned to bottom
- [x] 4.2 Manual test: change Tag/Message/Level filter with tail-following ON scrolls to newest visible entry
- [x] 4.3 Manual test: switch device with tail-following ON scrolls as new logs arrive
- [x] 4.4 Manual test: toggle OFF/ON, scroll up auto-disables, preference survives app restart
