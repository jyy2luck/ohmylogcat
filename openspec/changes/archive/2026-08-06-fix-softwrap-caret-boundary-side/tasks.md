## 1. Boundary-side model

- [x] 1.1 Define the Soft-Wrap boundary-side enum with previous-row and following-row variants.
- [x] 1.2 Add boundary-side state to the application caret without changing `LogPos` or `TextSelection`.
- [x] 1.3 Normalize or reset the side on caret clamping, direct placement, filtering, resizing, and Soft-Wrap toggles.

## 2. Side-aware display geometry

- [x] 2.1 Add side-aware helpers for resolving a logical boundary gap to its wrap chunk and display row.
- [x] 2.2 Add side-aware logical/display-column conversion, including previous-row right-edge gaps.
- [x] 2.3 Update caret screen mapping and visibility checks to use the side-aware geometry while preserving default mouse and selection mapping.

## 3. Keyboard movement

- [x] 3.1 Update Home and End to create the correct boundary side for the current wrap chunk.
- [x] 3.2 Implement the two-stage Right and Left transitions at internal Soft-Wrap boundaries.
- [x] 3.3 Thread boundary side through vertical movement and preferred display-column updates.
- [x] 3.4 Preserve Shift-selection semantics when a horizontal move changes only the visual side.

## 4. Regression tests

- [x] 4.1 Test End on non-final and final Soft-Wrap chunks, including the previous-row right-edge screen coordinate.
- [x] 4.2 Test the two-stage Right and symmetric Left boundary transitions.
- [x] 4.3 Test Home, Up/Down display-column tracking, clamp behavior, and caret visibility at both boundary sides.
- [x] 4.4 Test unchanged Soft-Wrap-off behavior, mouse placement, logical selection, and clipboard extraction.

## 5. Verification

- [x] 5.1 Run the full `cargo test` suite and confirm all tests pass.
- [x] 5.2 Validate the OpenSpec change and inspect the final diff for unrelated changes.
