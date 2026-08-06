mod display;
mod find;
mod format;
mod i18n;
mod pointer;
mod selection;
mod text_input;
mod theme;

pub use display::{
    effective_hang_indent, visible_chars, wrap_chunk_at_col_with_side, wrap_display_col_with_side,
    wrap_display_row_for_col_with_side, wrap_display_text, wrap_line_count,
    wrap_logical_pos_from_display, WrapCaretSide, WrapChunks,
};
pub use find::FindState;
pub use format::{format_log_line, message_column_indent};
pub use i18n::{LanguagePreference, Locale, UiStrings};
pub use pointer::{reset_pointer_shape, set_pointer_shape, PointerShape};
pub use selection::{
    clamp_log_pos, expand_line, expand_word, line_spans, log_pos_to_screen_with_side,
    mouse_to_log_pos, step_caret_horizontal, LogPos, TextSelection, ViewportMap,
};
pub use text_input::{str_display_width, TextInput, TEXT_INPUT_CURSOR_STYLE};
pub use theme::Theme;
