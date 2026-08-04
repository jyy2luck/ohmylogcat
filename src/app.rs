//! Terminal UI application: layout, focus, and controls over `Engine`.

use crate::adb::{self, Device};
use crate::engine::{Engine, EngineEvent};
use crate::parser::LogLevel;
use crate::settings::{
    load_settings, save_settings, BufferPreset, BufferStats, Settings,
};
use crate::ui::{
    clamp_log_pos, expand_line, expand_word, format_log_line, line_spans, log_pos_to_screen,
    message_column_indent, mouse_to_log_pos, reset_pointer_shape, set_pointer_shape,
    step_caret_horizontal, str_display_width, visible_chars, effective_hang_indent,
    wrap_chunk_at_col, wrap_display_col, wrap_display_row_for_col,
    wrap_display_text, wrap_line_count, wrap_logical_col_from_display,
    FindState, LanguagePreference, Locale, LogPos, PointerShape, TextInput, TextSelection, Theme,
    UiStrings, ViewportMap, WrapChunks, TEXT_INPUT_CURSOR_STYLE,
};
use crossterm::cursor::SetCursorStyle;
use crossterm::execute;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;
use std::io;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

const FILTER_DEBOUNCE: Duration = Duration::from_millis(200);
const DEFAULT_EXPORT_NAME: &str = "ohmylogcat.log";
const MULTI_CLICK_MS: u128 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Logs,
    Level,
    Find,
    Modal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterField {
    Tag,
    Message,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalKind {
    Devices,
    Export {
        filtered_only: bool,
        path: String,
    },
    Settings,
    ExportMenu,
    FilterEdit { field: FilterField },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsField {
    Adb,
    Preset,
    Custom,
    Language,
}

#[derive(Debug)]
pub struct SettingsPanelState {
    pub adb_path: String,
    /// Resolved auto path (PATH / platform defaults); shown as read-only hint when override empty.
    auto_adb: Option<String>,
    pub preset: BufferPreset,
    pub custom_capacity: String,
    pub language: LanguagePreference,
    pub status: Option<String>,
    focus_field: SettingsField,
    adb_editing: bool,
}

impl SettingsPanelState {
    pub fn from_settings(settings: &Settings) -> Self {
        let preset = BufferPreset::from_capacity(settings.buffer_capacity);
        Self {
            adb_path: settings.adb_path.clone().unwrap_or_default(),
            // ponytail: resolve once on open; skip per-frame where/which
            auto_adb: adb::resolve_adb_path(None).ok(),
            preset,
            custom_capacity: settings.buffer_capacity.to_string(),
            language: settings.language,
            status: None,
            focus_field: SettingsField::Adb,
            adb_editing: false,
        }
    }

    fn visible_fields(preset: BufferPreset) -> &'static [SettingsField] {
        static WITH_CUSTOM: [SettingsField; 4] = [
            SettingsField::Adb,
            SettingsField::Preset,
            SettingsField::Custom,
            SettingsField::Language,
        ];
        static WITHOUT_CUSTOM: [SettingsField; 3] = [
            SettingsField::Adb,
            SettingsField::Preset,
            SettingsField::Language,
        ];
        if preset == BufferPreset::Custom {
            &WITH_CUSTOM
        } else {
            &WITHOUT_CUSTOM
        }
    }

    fn move_focus(&mut self, delta: isize) {
        let fields = Self::visible_fields(self.preset);
        if !fields.contains(&self.focus_field) {
            self.focus_field = SettingsField::Preset;
            return;
        }
        let idx = fields
            .iter()
            .position(|f| *f == self.focus_field)
            .unwrap_or(0);
        let len = fields.len() as isize;
        let next = (idx as isize + delta).rem_euclid(len) as usize;
        let next_field = fields[next];
        if self.focus_field == SettingsField::Adb && next_field != SettingsField::Adb {
            self.adb_editing = false;
        }
        self.focus_field = next_field;
    }

    fn adb_is_custom(&self) -> bool {
        !self.adb_path.trim().is_empty()
    }

    fn adb_display_path(&self) -> String {
        if self.adb_is_custom() {
            self.adb_path.clone()
        } else {
            match &self.auto_adb {
                Some(path) => path.clone(),
                None => String::new(),
            }
        }
    }

    pub fn capacity(&self) -> usize {
        if self.preset == BufferPreset::Custom {
            self.custom_capacity.parse().unwrap_or(200_000)
        } else {
            self.preset.capacity().unwrap_or(200_000)
        }
    }
}

/// Hit regions for optional mouse clicks (updated each draw).
#[derive(Debug, Default, Clone)]
struct HitMap {
    toolbar: Vec<(Rect, ToolbarHit)>,
    filter_tag: Option<Rect>,
    filter_message: Option<Rect>,
    filter_level: Option<Rect>,
    filter_modal_input: Option<Rect>,
    find_input: Option<Rect>,
    log_viewport: Option<Rect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolbarHit {
    Devices,
    Pause,
    Clear,
    Follow,
    Wrap,
    Export,
    Settings,
}

pub struct OhmylogcatApp {
    _rt: Runtime,
    engine: Arc<Engine>,
    event_rx: Receiver<EngineEvent>,

    settings: Settings,
    theme: Theme,
    locale: Locale,
    ui: UiStrings,
    devices: Vec<Device>,
    selected_serial: Option<String>,
    device_cursor: usize,

    filter_tag: TextInput,
    filter_message: TextInput,
    filter_level: Option<LogLevel>,

    stats: BufferStats,
    last_error: Option<String>,
    status_message: Option<String>,
    /// When set, `status_message` is cleared after this instant (ephemeral tip).
    status_expires_at: Option<Instant>,

    auto_scroll: bool,
    soft_wrap: bool,
    scroll_offset: usize,
    /// When soft-wrap is on, skip this many wrapped lines of the first visible entry.
    wrap_skip: usize,
    col_offset: usize,
    viewport_height: usize,
    viewport_width: usize,

    focus: Focus,
    modal: Option<ModalKind>,
    settings_panel: SettingsPanelState,
    find: FindState,

    filter_dirty: bool,
    filter_changed_at: Option<Instant>,
    last_device_refresh: Instant,

    hit_map: HitMap,
    selection: TextSelection,
    /// Caret in the filtered log list; independent of selection endpoints.
    caret: Option<LogPos>,
    /// Preferred display column for Up/Down navigation.
    caret_preferred_col: usize,
    last_click_time: Option<Instant>,
    last_click_screen: Option<(u16, u16)>,
    click_count: u8,
    last_mouse: Option<(u16, u16)>,
    last_pointer: Option<PointerShape>,
    last_hardware_cursor_bar: Option<bool>,
    follow_dirty: bool,
    should_quit: bool,
}

impl OhmylogcatApp {
    pub fn new(keyboard_enhancement: bool) -> Self {
        let rt = Runtime::new().expect("tokio runtime");
        let settings = load_settings();
        let theme = Theme::resolve();
        let locale = Locale::resolve(settings.language);
        let ui = UiStrings::for_locale(locale);
        let (engine, event_rx) = Engine::new(settings.buffer_capacity);

        let mut app = Self {
            _rt: rt,
            engine,
            event_rx,
            devices: Vec::new(),
            selected_serial: None,
            device_cursor: 0,
            filter_tag: TextInput::new(),
            filter_message: TextInput::new(),
            filter_level: None,
            stats: BufferStats {
                capacity: settings.buffer_capacity,
                ..Default::default()
            },
            last_error: None,
            status_message: None,
            status_expires_at: None,
            auto_scroll: settings.auto_scroll_to_end,
            soft_wrap: settings.soft_wrap,
            scroll_offset: 0,
            wrap_skip: 0,
            col_offset: 0,
            viewport_height: 10,
            viewport_width: 80,
            focus: Focus::Logs,
            modal: None,
            settings_panel: SettingsPanelState::from_settings(&settings),
            find: FindState::default(),
            filter_dirty: false,
            filter_changed_at: None,
            last_device_refresh: Instant::now() - Duration::from_secs(60),
            hit_map: HitMap::default(),
            selection: TextSelection::default(),
            caret: None,
            caret_preferred_col: 0,
            last_click_time: None,
            last_click_screen: None,
            click_count: 0,
            last_mouse: None,
            last_pointer: None,
            last_hardware_cursor_bar: None,
            follow_dirty: true,
            should_quit: false,
            settings,
            theme,
            locale,
            ui,
        };
        if !keyboard_enhancement {
            app.set_ephemeral_status(
                app.ui.tip_keyboard_enhancement.into(),
                Duration::from_secs(8),
            );
        }
        app.refresh_devices();
        app
    }

    fn set_ephemeral_status(&mut self, message: String, duration: Duration) {
        self.status_message = Some(message);
        self.status_expires_at = Some(Instant::now() + duration);
    }

    fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
        self.status_expires_at = None;
    }

    fn runtime_handle(&self) -> tokio::runtime::Handle {
        self._rt.handle().clone()
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn tick(&mut self) {
        self.drain_events();

        if let Some(until) = self.status_expires_at {
            if Instant::now() >= until {
                self.status_message = None;
                self.status_expires_at = None;
            }
        }

        if self.filter_dirty {
            let due = self
                .filter_changed_at
                .map(|t| t.elapsed() >= FILTER_DEBOUNCE)
                .unwrap_or(true);
            if due {
                self.apply_filter();
                self.filter_dirty = false;
                self.filter_changed_at = None;
            }
        }

        if self.last_device_refresh.elapsed() > Duration::from_secs(5) {
            self.refresh_devices();
        }

        self.clamp_scroll();
        self.seed_or_clamp_caret();
        // Follow scroll is applied in `draw_logs` after viewport size is known.
        // Doing it here used the previous frame's (or default) height/width and
        // left Wrap mode stuck on the start of a long line.
        if let Some(row) = self.find.scroll_to_match.take() {
            self.ensure_row_visible(row);
        }

        self.sync_terminal_cursor_style();
    }

    pub fn handle_event(&mut self, event: Event) -> io::Result<()> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat => {
                self.handle_key(key);
            }
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            Event::Resize(_, _) => {
                self.follow_dirty = true;
            }
            _ => {}
        }
        Ok(())
    }

    fn is_top_layer(&self) -> bool {
        self.modal.is_none() && !self.find.open
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if is_copy_shortcut(&key) && self.copy_selection() {
            return;
        }

        if self.is_top_layer()
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q'))
            && !key.modifiers.contains(KeyModifiers::CONTROL)
        {
            self.should_quit = true;
            return;
        }

        if self.modal.is_some() {
            self.handle_modal_key(key);
            return;
        }

        match self.focus {
            Focus::Logs => self.handle_logs_key(key),
            Focus::Level => self.handle_level_key(key),
            Focus::Find => self.handle_find_key(key),
            Focus::Modal => self.handle_modal_key(key),
        }
    }

    fn handle_logs_key(&mut self, key: KeyEvent) {
        // Ctrl/Cmd+F
        if key.code == KeyCode::Char('f')
            && (key.modifiers.contains(KeyModifiers::CONTROL)
                || key.modifiers.contains(KeyModifiers::SUPER))
        {
            self.open_find();
            return;
        }

        match key.code {
            KeyCode::Char(' ') => self.toggle_pause(),
            KeyCode::Char('c')
                if !key.modifiers.intersects(
                    KeyModifiers::CONTROL | KeyModifiers::SUPER | KeyModifiers::META,
                ) =>
            {
                self.clear_logs();
            }
            KeyCode::Char('f') => self.toggle_follow(),
            KeyCode::Char('d') => self.open_devices(),
            KeyCode::Char('e') => {
                self.modal = Some(ModalKind::ExportMenu);
                self.focus = Focus::Modal;
            }
            KeyCode::Char('s') => self.open_settings(),
            KeyCode::Char('w') => self.toggle_wrap(),
            KeyCode::Char('/') => self.open_find(),
            KeyCode::Char('n') if self.find.open => self.find.next(),
            KeyCode::Char('N') if self.find.open => self.find.prev(),
            KeyCode::Char('t') if self.is_top_layer() => {
                self.open_filter_edit(FilterField::Tag);
            }
            KeyCode::Char('m') if self.is_top_layer() => {
                self.open_filter_edit(FilterField::Message);
            }
            KeyCode::Char('l') => {
                self.focus = Focus::Level;
            }
            KeyCode::Tab => self.focus = Focus::Level,
            KeyCode::Up => self.move_caret_vertical(-1, key.modifiers.contains(KeyModifiers::SHIFT)),
            KeyCode::Down => self.move_caret_vertical(1, key.modifiers.contains(KeyModifiers::SHIFT)),
            KeyCode::Left => self.move_caret_horizontal(-1, key.modifiers.contains(KeyModifiers::SHIFT)),
            KeyCode::Right => self.move_caret_horizontal(1, key.modifiers.contains(KeyModifiers::SHIFT)),
            KeyCode::PageUp => {
                self.move_caret_vertical(
                    -(self.viewport_height.max(1) as isize),
                    key.modifiers.contains(KeyModifiers::SHIFT),
                );
            }
            KeyCode::PageDown => {
                self.move_caret_vertical(
                    self.viewport_height.max(1) as isize,
                    key.modifiers.contains(KeyModifiers::SHIFT),
                );
            }
            KeyCode::Home => self.move_caret_line_bound(true, key.modifiers.contains(KeyModifiers::SHIFT)),
            KeyCode::End => self.move_caret_line_bound(false, key.modifiers.contains(KeyModifiers::SHIFT)),
            KeyCode::Esc if self.find.open => {
                self.find.close();
                self.focus = Focus::Logs;
            }
            _ => {}
        }
    }

    fn handle_level_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => self.focus = Focus::Logs,
            KeyCode::Tab => self.focus = Focus::Logs,
            KeyCode::BackTab => self.focus = Focus::Logs,
            KeyCode::Left | KeyCode::Up | KeyCode::Char('h') | KeyCode::Char('k') => {
                self.cycle_level(false);
            }
            KeyCode::Right | KeyCode::Down | KeyCode::Char('l') | KeyCode::Char('j') | KeyCode::Char(' ') => {
                self.cycle_level(true);
            }
            _ => {}
        }
    }

    fn handle_find_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.find.close();
                self.focus = Focus::Logs;
            }
            KeyCode::Enter => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.find.prev();
                } else {
                    self.find.next();
                }
            }
            _ => {
                if self.find.input.handle_key(key) {
                    self.find.recompute(&self.engine);
                }
            }
        }
    }

    fn handle_modal_key(&mut self, key: KeyEvent) {
        let Some(kind) = self.modal.clone() else {
            return;
        };
        match kind {
            ModalKind::Devices => self.handle_devices_modal_key(key),
            ModalKind::Export { .. } => self.handle_export_modal_key(key),
            ModalKind::Settings => self.handle_settings_modal_key(key),
            ModalKind::ExportMenu => self.handle_export_menu_key(key),
            ModalKind::FilterEdit { field } => self.handle_filter_edit_modal_key(key, field),
        }
    }

    fn handle_filter_edit_modal_key(&mut self, key: KeyEvent, field: FilterField) {
        match key.code {
            KeyCode::Enter => self.close_modal(),
            KeyCode::Esc => {
                let input = match field {
                    FilterField::Tag => &mut self.filter_tag,
                    FilterField::Message => &mut self.filter_message,
                };
                if input.text.is_empty() {
                    self.close_modal();
                } else {
                    input.text.clear();
                    input.cursor = 0;
                    self.mark_filter_dirty();
                }
            }
            _ => {
                let input = match field {
                    FilterField::Tag => &mut self.filter_tag,
                    FilterField::Message => &mut self.filter_message,
                };
                if input.handle_key(key) {
                    self.mark_filter_dirty();
                }
            }
        }
    }

    fn handle_devices_modal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.close_modal(),
            KeyCode::Up | KeyCode::Char('k') => {
                if self.device_cursor > 0 {
                    self.device_cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.devices.len(); // +1 for (none)
                if self.device_cursor < max {
                    self.device_cursor += 1;
                }
            }
            KeyCode::Char('r') => self.refresh_devices(),
            KeyCode::Enter => {
                if self.device_cursor == 0 {
                    self.selected_serial = None;
                    self.engine.stop_stream();
                } else if let Some(dev) = self.devices.get(self.device_cursor - 1) {
                    self.selected_serial = Some(dev.serial.clone());
                    self.start_selected_device();
                }
                self.close_modal();
            }
            _ => {}
        }
    }

    fn handle_export_menu_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.close_modal(),
            KeyCode::Char('1') | KeyCode::Char('f') => {
                self.open_export(true);
            }
            KeyCode::Char('2') | KeyCode::Char('a') => {
                self.open_export(false);
            }
            _ => {}
        }
    }

    fn handle_export_modal_key(&mut self, key: KeyEvent) {
        let Some(ModalKind::Export {
            filtered_only,
            mut path,
        }) = self.modal.clone()
        else {
            return;
        };
        match key.code {
            KeyCode::Esc => self.close_modal(),
            KeyCode::Enter => {
                match self.engine.export_to_file(&path, filtered_only) {
                    Ok(()) => {
                        self.set_status(
                            self.ui.status_exported_to.replace("{}", &path),
                        );
                        self.last_error = None;
                    }
                    Err(e) => self.last_error = Some(e),
                }
                self.close_modal();
            }
            KeyCode::Backspace => {
                path.pop();
                self.modal = Some(ModalKind::Export {
                    filtered_only,
                    path,
                });
            }
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::SUPER) =>
            {
                path.push(c);
                self.modal = Some(ModalKind::Export {
                    filtered_only,
                    path,
                });
            }
            _ => {}
        }
    }

    fn handle_settings_modal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => self.close_modal(),
            KeyCode::Esc => {
                if self.settings_panel.focus_field == SettingsField::Adb
                    && self.settings_panel.adb_editing
                {
                    self.settings_panel.adb_editing = false;
                } else {
                    self.close_modal();
                }
            }
            KeyCode::Up => self.settings_panel.move_focus(-1),
            KeyCode::Down => self.settings_panel.move_focus(1),
            KeyCode::Left | KeyCode::Right
                if self.settings_panel.focus_field == SettingsField::Preset =>
            {
                let forward = key.code == KeyCode::Right;
                self.cycle_preset(forward);
            }
            KeyCode::Left | KeyCode::Right
                if self.settings_panel.focus_field == SettingsField::Language =>
            {
                let forward = key.code == KeyCode::Right;
                self.cycle_language(forward);
            }
            KeyCode::Backspace => {
                match self.settings_panel.focus_field {
                    SettingsField::Adb if self.settings_panel.adb_editing => {
                        self.settings_panel.adb_path.pop();
                        self.commit_settings_from_panel();
                    }
                    SettingsField::Custom => {
                        self.settings_panel.custom_capacity.pop();
                        self.commit_settings_from_panel();
                    }
                    _ => {}
                }
            }
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !matches!(
                        self.settings_panel.focus_field,
                        SettingsField::Preset | SettingsField::Language
                    ) =>
            {
                match self.settings_panel.focus_field {
                    SettingsField::Adb => {
                        if self.settings_panel.adb_editing {
                            self.settings_panel.adb_path.push(c);
                            self.commit_settings_from_panel();
                        } else if c == 'e' {
                            self.settings_panel.adb_editing = true;
                        } else if c == 'r' {
                            self.settings_panel.adb_path.clear();
                            self.commit_settings_from_panel();
                        }
                    }
                    SettingsField::Custom => {
                        if c.is_ascii_digit() {
                            self.settings_panel.custom_capacity.push(c);
                            self.commit_settings_from_panel();
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        let col = mouse.column;
        let row = mouse.row;
        self.note_mouse(col, row);

        match mouse.kind {
            MouseEventKind::Moved => {}
            MouseEventKind::ScrollUp => self.scroll_by(-3),
            MouseEventKind::ScrollDown => self.scroll_by(3),
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(r) = self.hit_map.filter_modal_input {
                    if contains(r, col, row) {
                        if let Some(ModalKind::FilterEdit { field }) = self.modal {
                            let input = match field {
                                FilterField::Tag => &mut self.filter_tag,
                                FilterField::Message => &mut self.filter_message,
                            };
                            input.cursor =
                                TextInput::cursor_from_click(col, r.x, &input.text);
                            self.focus = Focus::Modal;
                            return;
                        }
                    }
                }
                if let Some(r) = self.hit_map.find_input {
                    if contains(r, col, row) {
                        self.find.input.cursor =
                            TextInput::cursor_from_click(col, r.x, &self.find.input.text);
                        self.focus = Focus::Find;
                        return;
                    }
                }
                if self.try_start_log_selection(col, row) {
                    return;
                }
                self.selection.clear();
                for (rect, hit) in &self.hit_map.toolbar {
                    if contains(*rect, col, row) {
                        match hit {
                            ToolbarHit::Devices => self.open_devices(),
                            ToolbarHit::Pause => self.toggle_pause(),
                            ToolbarHit::Clear => self.clear_logs(),
                            ToolbarHit::Follow => self.toggle_follow(),
                            ToolbarHit::Wrap => self.toggle_wrap(),
                            ToolbarHit::Export => {
                                self.modal = Some(ModalKind::ExportMenu);
                                self.focus = Focus::Modal;
                            }
                            ToolbarHit::Settings => self.open_settings(),
                        }
                        return;
                    }
                }
                if let Some(r) = self.hit_map.filter_tag {
                    if contains(r, col, row) {
                        self.open_filter_edit(FilterField::Tag);
                        return;
                    }
                }
                if let Some(r) = self.hit_map.filter_message {
                    if contains(r, col, row) {
                        self.open_filter_edit(FilterField::Message);
                        return;
                    }
                }
                if let Some(r) = self.hit_map.filter_level {
                    if contains(r, col, row) {
                        self.focus = Focus::Level;
                        return;
                    }
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if let Some(pos) = self.mouse_to_log_pos(col, row) {
                    if self.selection.dragging() {
                        self.selection.extend_to(pos);
                        self.caret = Some(pos);
                        self.caret_preferred_col = self.display_col_of(pos);
                    } else if self.hit_map.log_viewport.is_some_and(|r| contains(r, col, row)) {
                        // Only start a fresh drag from empty selection; keep
                        // word/line multi-click ranges intact on micro-moves.
                        if !self.selection.has_extent() {
                            self.selection.start(pos);
                            self.caret = Some(pos);
                            self.caret_preferred_col = self.display_col_of(pos);
                            self.focus = Focus::Logs;
                        }
                    }
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.selection.finish_drag();
                if !self.selection.has_extent() {
                    self.selection.clear();
                }
            }
            _ => {}
        }
    }

    fn note_mouse(&mut self, col: u16, row: u16) {
        self.last_mouse = Some((col, row));
    }

    fn try_start_log_selection(&mut self, col: u16, row: u16) -> bool {
        let Some(r) = self.hit_map.log_viewport else {
            return false;
        };
        if !contains(r, col, row) {
            return false;
        }
        let Some(pos) = self.mouse_to_log_pos(col, row) else {
            return false;
        };
        self.focus = Focus::Logs;
        let count = self.register_multi_click(col, row);
        match count {
            2 => {
                self.select_word_at(pos);
            }
            3 => {
                self.select_line_at(pos);
            }
            _ => {
                self.caret = Some(pos);
                self.caret_preferred_col = self.display_col_of(pos);
                self.selection.clear();
                self.selection.start(pos);
            }
        }
        true
    }

    fn register_multi_click(&mut self, col: u16, row: u16) -> u8 {
        let now = Instant::now();
        let near = self
            .last_click_screen
            .is_some_and(|(c, r)| c.abs_diff(col) <= 1 && r.abs_diff(row) <= 1);
        let quick = self
            .last_click_time
            .is_some_and(|t| now.duration_since(t).as_millis() <= MULTI_CLICK_MS);
        if near && quick {
            self.click_count = match self.click_count {
                1 => 2,
                2 => 3,
                _ => 1,
            };
        } else {
            self.click_count = 1;
        }
        self.last_click_time = Some(now);
        self.last_click_screen = Some((col, row));
        self.click_count
    }

    fn select_word_at(&mut self, pos: LogPos) {
        let Some(line) = self.formatted_line_at(pos.row) else {
            return;
        };
        let (start, end) = expand_word(&line, pos.col);
        let anchor = LogPos {
            row: pos.row,
            col: start,
        };
        let cursor = LogPos {
            row: pos.row,
            col: end,
        };
        self.selection.set_range(anchor, cursor);
        self.caret = Some(cursor);
        self.caret_preferred_col = self.display_col_of(cursor);
    }

    fn select_line_at(&mut self, pos: LogPos) {
        let Some(line) = self.formatted_line_at(pos.row) else {
            return;
        };
        let (start, end) = expand_line(&line);
        let anchor = LogPos {
            row: pos.row,
            col: start,
        };
        let cursor = LogPos {
            row: pos.row,
            col: end,
        };
        self.selection.set_range(anchor, cursor);
        self.caret = Some(cursor);
        self.caret_preferred_col = self.display_col_of(cursor);
    }

    fn mouse_to_log_pos(&self, col: u16, row: u16) -> Option<LogPos> {
        let area = self.hit_map.log_viewport?;
        let row_count = self.engine.filtered_len();
        let map = self.viewport_map(area);
        mouse_to_log_pos(
            col,
            row,
            &map,
            |idx| self.formatted_line_at(idx),
            row_count,
        )
    }

    fn viewport_map(&self, area: Rect) -> ViewportMap {
        ViewportMap {
            area,
            scroll_offset: self.scroll_offset,
            wrap_skip: self.wrap_skip,
            col_offset: self.col_offset,
            soft_wrap: self.soft_wrap,
            viewport_width: self.viewport_width,
            viewport_height: self.viewport_height,
        }
    }

    fn formatted_line_at(&self, idx: usize) -> Option<String> {
        self.engine.filtered_get(idx).map(|e| format_log_line(&e))
    }

    fn copy_selection(&mut self) -> bool {
        if !self.selection.has_extent() {
            return false;
        }
        let Some(text) = self.selection.extract_text(|idx| self.formatted_line_at(idx)) else {
            return false;
        };
        match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(text)) {
            Ok(()) => {
                self.set_status(self.ui.status_copied.into());
                true
            }
            Err(e) => {
                self.last_error = Some(self.ui.status_copy_failed.replace("{}", &e.to_string()));
                true
            }
        }
    }

    pub fn apply_pointer_shape(&mut self) {
        let Some((col, row)) = self.last_mouse else {
            return;
        };
        let shape = if self
            .hit_map
            .filter_modal_input
            .is_some_and(|r| contains(r, col, row))
            || self
                .hit_map
                .find_input
                .is_some_and(|r| contains(r, col, row))
        {
            PointerShape::Text
        } else if self.modal.is_some() {
            PointerShape::Default
        } else if self
            .hit_map
            .log_viewport
            .is_some_and(|r| contains(r, col, row))
        {
            PointerShape::Text
        } else {
            PointerShape::Default
        };
        if self.last_pointer == Some(shape) {
            return;
        }
        if set_pointer_shape(shape).is_ok() {
            self.last_pointer = Some(shape);
        }
    }

    fn sync_terminal_cursor_style(&mut self) {
        let bar = self.hardware_cursor_bar_active();
        if self.last_hardware_cursor_bar == Some(bar) {
            return;
        }
        self.last_hardware_cursor_bar = Some(bar);
        let style = if bar {
            TEXT_INPUT_CURSOR_STYLE
        } else {
            SetCursorStyle::DefaultUserShape
        };
        let _ = execute!(io::stdout(), style);
    }

    fn hardware_cursor_bar_active(&self) -> bool {
        if self.text_input_focused() {
            return true;
        }
        self.log_caret_screen_pos().is_some()
    }

    fn text_input_focused(&self) -> bool {
        match self.focus {
            Focus::Find if self.find.open => true,
            Focus::Modal => matches!(self.modal, Some(ModalKind::FilterEdit { .. })),
            _ => false,
        }
    }

    fn log_caret_screen_pos(&self) -> Option<(u16, u16)> {
        if self.focus != Focus::Logs || self.text_input_focused() {
            return None;
        }
        let caret = self.caret?;
        let area = self.hit_map.log_viewport?;
        let row_count = self.engine.filtered_len();
        if row_count == 0 {
            return None;
        }
        let map = self.viewport_map(area);
        log_pos_to_screen(caret, &map, |idx| self.formatted_line_at(idx), row_count)
    }

    pub fn restore_pointer(&mut self) {
        reset_pointer_shape();
        self.last_pointer = None;
    }

    // --- actions ---

    fn open_find(&mut self) {
        self.find.open_bar();
        self.focus = Focus::Find;
    }

    fn open_devices(&mut self) {
        self.refresh_devices();
        self.device_cursor = match &self.selected_serial {
            None => 0,
            Some(s) => self
                .devices
                .iter()
                .position(|d| &d.serial == s)
                .map(|i| i + 1)
                .unwrap_or(0),
        };
        self.modal = Some(ModalKind::Devices);
        self.focus = Focus::Modal;
    }

    fn open_settings(&mut self) {
        self.settings_panel = SettingsPanelState::from_settings(&self.settings);
        self.modal = Some(ModalKind::Settings);
        self.focus = Focus::Modal;
    }

    fn open_export(&mut self, filtered_only: bool) {
        self.modal = Some(ModalKind::Export {
            filtered_only,
            path: DEFAULT_EXPORT_NAME.into(),
        });
        self.focus = Focus::Modal;
    }

    fn open_filter_edit(&mut self, field: FilterField) {
        match field {
            FilterField::Tag => self.filter_tag.set_cursor_end(),
            FilterField::Message => self.filter_message.set_cursor_end(),
        }
        self.modal = Some(ModalKind::FilterEdit { field });
        self.focus = Focus::Modal;
    }

    fn close_modal(&mut self) {
        self.modal = None;
        self.focus = Focus::Logs;
    }

    fn toggle_pause(&mut self) {
        if self.engine.is_paused() {
            self.engine.resume();
        } else {
            self.engine.pause();
        }
    }

    fn clear_logs(&mut self) {
        self.engine.clear_buffer();
        self.scroll_offset = 0;
        self.col_offset = 0;
        self.wrap_skip = 0;
        self.selection.clear();
        self.caret = None;
        self.caret_preferred_col = 0;
        if self.find.open {
            self.find.recompute(&self.engine);
        }
    }

    fn seed_or_clamp_caret(&mut self) {
        let n = self.engine.filtered_len();
        if n == 0 {
            self.caret = None;
            return;
        }
        let seeding = self.caret.is_none();
        let pos = match self.caret {
            Some(c) => c,
            None => LogPos {
                row: self.scroll_offset.min(n - 1),
                col: 0,
            },
        };
        self.caret = clamp_log_pos(pos, n, |idx| self.formatted_line_at(idx));
        if seeding {
            if let Some(c) = self.caret {
                self.caret_preferred_col = self.display_col_of(c);
            }
        }
    }

    fn display_col_of(&self, pos: LogPos) -> usize {
        if self.soft_wrap {
            let line = self.formatted_line_at(pos.row).unwrap_or_default();
            let width = self.viewport_width.max(1);
            let indent = self.entry_indent_at(pos.row);
            wrap_display_col(&line, width, indent, pos.col)
        } else {
            pos.col
        }
    }

    fn line_len_at(&self, row: usize) -> usize {
        self.formatted_line_at(row)
            .map(|s| s.chars().count())
            .unwrap_or(0)
    }

    fn clamp_col_for_row(&self, row: usize, preferred: usize) -> usize {
        let len = self.line_len_at(row);
        if len == 0 {
            0
        } else {
            preferred.min(len - 1)
        }
    }

    fn apply_caret_move(&mut self, old: LogPos, new: LogPos, extend: bool) {
        if extend {
            if !self.selection.has_extent() {
                self.selection.set_range(old, new);
            } else {
                self.selection.extend_to(new);
            }
        } else {
            self.selection.clear();
        }
        self.caret = Some(new);
        self.caret_preferred_col = self.display_col_of(new);
        self.ensure_caret_visible();
        self.disable_follow_if_needed();
    }

    fn move_caret_horizontal(&mut self, delta: isize, extend: bool) {
        self.seed_or_clamp_caret();
        let Some(old) = self.caret else {
            return;
        };
        let n = self.engine.filtered_len();
        let new = step_caret_horizontal(old, delta, n, |row| self.line_len_at(row));
        if new != old || extend {
            self.apply_caret_move(old, new, extend);
        } else {
            self.ensure_caret_visible();
        }
    }

    fn move_caret_line_bound(&mut self, home: bool, extend: bool) {
        self.seed_or_clamp_caret();
        let Some(old) = self.caret else {
            return;
        };
        let col = if home {
            0
        } else {
            self.line_len_at(old.row).saturating_sub(1)
        };
        let new = LogPos { row: old.row, col };
        self.apply_caret_move(old, new, extend);
    }

    fn move_caret_vertical(&mut self, delta_rows: isize, extend: bool) {
        self.seed_or_clamp_caret();
        let Some(old) = self.caret else {
            return;
        };
        let preferred = self.caret_preferred_col;
        let new = if self.soft_wrap {
            self.move_caret_vertical_wrapped(old, delta_rows, preferred)
        } else {
            self.move_caret_vertical_nowrap(old, delta_rows, preferred)
        };
        // Preserve preferred column across short lines.
        self.caret_preferred_col = preferred;
        self.apply_caret_move(old, new, extend);
        self.caret_preferred_col = preferred;
    }

    fn move_caret_vertical_nowrap(&self, old: LogPos, delta_rows: isize, preferred: usize) -> LogPos {
        let n = self.engine.filtered_len();
        if n == 0 {
            return old;
        }
        let row = if delta_rows < 0 {
            old.row.saturating_sub((-delta_rows) as usize)
        } else {
            (old.row + delta_rows as usize).min(n - 1)
        };
        LogPos {
            row,
            col: self.clamp_col_for_row(row, preferred),
        }
    }

    fn move_caret_vertical_wrapped(
        &self,
        old: LogPos,
        delta_rows: isize,
        preferred: usize,
    ) -> LogPos {
        let n = self.engine.filtered_len();
        if n == 0 || delta_rows == 0 {
            return old;
        }
        let width = self.viewport_width.max(1);
        let mut row = old.row;
        let line = self.formatted_line_at(row).unwrap_or_default();
        let indent = self.entry_indent_at(row);
        let (mut chunk, _, _) = wrap_chunk_at_col(&line, width, indent, old.col);
        if delta_rows < 0 {
            let mut left = (-delta_rows) as usize;
            while left > 0 {
                if chunk > 0 {
                    chunk -= 1;
                    left -= 1;
                } else if row > 0 {
                    row -= 1;
                    let h = self.entry_wrap_height(row, width).max(1);
                    chunk = h.saturating_sub(1);
                    left -= 1;
                } else {
                    break;
                }
            }
        } else {
            let mut left = delta_rows as usize;
            while left > 0 {
                let h = self.entry_wrap_height(row, width).max(1);
                if chunk + 1 < h {
                    chunk += 1;
                    left -= 1;
                } else if row + 1 < n {
                    row += 1;
                    chunk = 0;
                    left -= 1;
                } else {
                    break;
                }
            }
        }
        let line_len = self.line_len_at(row);
        let col = if line_len == 0 {
            0
        } else {
            let line = self.formatted_line_at(row).unwrap_or_default();
            let indent = self.entry_indent_at(row);
            wrap_logical_col_from_display(&line, width, indent, chunk, preferred)
        };
        LogPos { row, col }
    }

    fn ensure_caret_visible(&mut self) {
        let Some(caret) = self.caret else {
            return;
        };
        let n = self.engine.filtered_len();
        if n == 0 {
            return;
        }

        if self.log_pos_vertically_visible(caret) {
            // ok
        } else if self.caret_is_above_viewport(caret) {
            if self.soft_wrap {
                let width = self.viewport_width.max(1);
                self.scroll_offset = caret.row;
                let line = self.formatted_line_at(caret.row).unwrap_or_default();
                let indent = self.entry_indent_at(caret.row);
                self.wrap_skip = wrap_display_row_for_col(&line, width, indent, caret.col);
            } else {
                self.scroll_offset = caret.row;
            }
        } else {
            // Below viewport: place caret at bottom of view.
            if self.soft_wrap {
                let width = self.viewport_width.max(1);
                self.scroll_offset = caret.row;
                let line = self.formatted_line_at(caret.row).unwrap_or_default();
                let indent = self.entry_indent_at(caret.row);
                self.wrap_skip = wrap_display_row_for_col(&line, width, indent, caret.col);
                let up = (self.viewport_height.max(1) - 1) as isize;
                self.scroll_by_wrapped(-up);
            } else {
                let h = self.viewport_height.max(1);
                self.scroll_offset = caret.row.saturating_add(1).saturating_sub(h);
            }
        }
        self.clamp_scroll();

        if !self.soft_wrap {
            let w = self.viewport_width.max(1);
            if caret.col < self.col_offset {
                self.col_offset = caret.col;
            } else if caret.col >= self.col_offset + w {
                self.col_offset = caret.col + 1 - w;
            }
        }
    }

    fn caret_is_above_viewport(&self, caret: LogPos) -> bool {
        if caret.row < self.scroll_offset {
            return true;
        }
        if caret.row > self.scroll_offset {
            return false;
        }
        if self.soft_wrap {
            let width = self.viewport_width.max(1);
            let line = self.formatted_line_at(caret.row).unwrap_or_default();
            let indent = self.entry_indent_at(caret.row);
            wrap_display_row_for_col(&line, width, indent, caret.col) < self.wrap_skip
        } else {
            false
        }
    }

    fn log_pos_vertically_visible(&self, pos: LogPos) -> bool {
        let Some(area) = self.hit_map.log_viewport else {
            // Viewport unknown yet — treat nowrap row window as authority.
            if self.soft_wrap {
                return pos.row == self.scroll_offset;
            }
            let h = self.viewport_height.max(1);
            return pos.row >= self.scroll_offset && pos.row < self.scroll_offset + h;
        };
        let row_count = self.engine.filtered_len();
        let map = self.viewport_map(area);
        // Visible if any screen mapping exists ignoring horizontal pan.
        if self.soft_wrap {
            log_pos_to_screen(pos, &map, |idx| self.formatted_line_at(idx), row_count).is_some()
                || self.log_pos_on_screen_vertically_wrapped(pos)
        } else {
            pos.row >= self.scroll_offset
                && pos.row < self.scroll_offset + self.viewport_height.max(1)
        }
    }

    fn log_pos_on_screen_vertically_wrapped(&self, pos: LogPos) -> bool {
        let width = self.viewport_width.max(1);
        let row_count = self.engine.filtered_len();
        let mut display_row = 0usize;
        let mut idx = self.scroll_offset;
        let mut skip = self.wrap_skip;
        while idx < row_count && display_row < self.viewport_height {
            let h = self.entry_wrap_height(idx, width).max(1);
            let start_chunk = skip;
            let chunks_shown = h.saturating_sub(start_chunk);
            if idx == pos.row {
                let line = self.formatted_line_at(idx).unwrap_or_default();
                let indent = self.entry_indent_at(idx);
                let chunk = wrap_display_row_for_col(&line, width, indent, pos.col);
                if chunk >= start_chunk
                    && chunk < start_chunk + chunks_shown.min(self.viewport_height - display_row)
                {
                    return true;
                }
                return false;
            }
            display_row += chunks_shown.min(self.viewport_height - display_row);
            skip = 0;
            idx += 1;
        }
        false
    }

    fn toggle_follow(&mut self) {
        self.auto_scroll = !self.auto_scroll;
        if self.auto_scroll {
            self.follow_dirty = true;
        }
        self.persist_display_prefs();
    }

    fn toggle_wrap(&mut self) {
        self.soft_wrap = !self.soft_wrap;
        if self.soft_wrap {
            self.col_offset = 0;
            self.wrap_skip = 0;
        } else {
            self.wrap_skip = 0;
        }
        self.persist_display_prefs();
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    fn disable_follow_if_needed(&mut self) {
        if self.auto_scroll && !self.find.is_active_with_matches() {
            self.auto_scroll = false;
            self.persist_display_prefs();
        }
    }

    fn scroll_by(&mut self, delta: isize) {
        if self.soft_wrap {
            self.scroll_by_wrapped(delta);
            return;
        }
        if delta < 0 {
            let d = (-delta) as usize;
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(d);
                self.disable_follow_if_needed();
            }
        } else {
            self.scroll_offset = self.scroll_offset.saturating_add(delta as usize);
            self.clamp_scroll();
            let max = self.max_scroll();
            if self.scroll_offset < max {
                self.disable_follow_if_needed();
            }
        }
    }

    /// Scroll by display lines when soft-wrap is on.
    fn scroll_by_wrapped(&mut self, delta: isize) {
        if delta == 0 {
            return;
        }
        let width = self.viewport_width.max(1);
        let n = self.engine.filtered_len();
        if n == 0 {
            self.scroll_offset = 0;
            self.wrap_skip = 0;
            return;
        }

        if delta < 0 {
            let mut remaining = (-delta) as usize;
            while remaining > 0 {
                if self.wrap_skip > 0 {
                    let step = self.wrap_skip.min(remaining);
                    self.wrap_skip -= step;
                    remaining -= step;
                } else if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                    let h = self.entry_wrap_height(self.scroll_offset, width);
                    self.wrap_skip = h.saturating_sub(1);
                    remaining -= 1;
                } else {
                    break;
                }
            }
            self.disable_follow_if_needed();
        } else {
            let mut remaining = delta as usize;
            while remaining > 0 {
                let h = self.entry_wrap_height(self.scroll_offset, width);
                let lines_after_skip = h.saturating_sub(self.wrap_skip);
                if lines_after_skip > 1 {
                    self.wrap_skip += 1;
                    remaining -= 1;
                } else if self.scroll_offset + 1 < n {
                    self.scroll_offset += 1;
                    self.wrap_skip = 0;
                    remaining -= 1;
                } else {
                    // At bottom
                    break;
                }
            }
            if !self.is_at_wrapped_bottom() {
                self.disable_follow_if_needed();
            }
        }
    }

    fn entry_wrap_height(&self, index: usize, width: usize) -> usize {
        match self.engine.filtered_get(index) {
            Some(e) => {
                let line = format_log_line(&e);
                let indent = message_column_indent(&e);
                wrap_line_count(&line, width, indent)
            }
            None => 1,
        }
    }

    fn entry_indent_at(&self, index: usize) -> usize {
        self.engine
            .filtered_get(index)
            .map(|e| message_column_indent(&e))
            .unwrap_or(0)
    }

    fn is_at_wrapped_bottom(&self) -> bool {
        let n = self.engine.filtered_len();
        if n == 0 {
            return true;
        }
        let width = self.viewport_width.max(1);
        let height = self.viewport_height.max(1);
        let mut idx = n - 1;
        let mut lines_after = self.entry_wrap_height(idx, width);
        while lines_after < height && idx > 0 {
            idx -= 1;
            lines_after += self.entry_wrap_height(idx, width);
        }
        let want_offset = idx;
        let want_skip = lines_after.saturating_sub(height);
        self.scroll_offset == want_offset && self.wrap_skip == want_skip
    }

    fn scroll_to_bottom(&mut self) {
        let n = self.engine.filtered_len();
        if n == 0 {
            self.scroll_offset = 0;
            self.wrap_skip = 0;
            return;
        }
        if !self.soft_wrap {
            self.scroll_offset = self.max_scroll();
            self.wrap_skip = 0;
            return;
        }

        let width = self.viewport_width.max(1);
        let height = self.viewport_height.max(1);
        let mut idx = n - 1;
        let mut lines_after = self.entry_wrap_height(idx, width).max(1);
        while lines_after < height && idx > 0 {
            idx -= 1;
            lines_after += self.entry_wrap_height(idx, width).max(1);
        }
        self.scroll_offset = idx;
        self.wrap_skip = lines_after.saturating_sub(height);
    }

    fn apply_follow_scroll_if_needed(&mut self) {
        if self.auto_scroll && !self.find.is_active_with_matches() && self.follow_dirty {
            self.scroll_to_bottom();
            self.follow_dirty = false;
        }
    }

    fn max_scroll(&self) -> usize {
        let n = self.engine.filtered_len();
        if self.soft_wrap {
            n.saturating_sub(1)
        } else {
            n.saturating_sub(self.viewport_height.max(1))
        }
    }

    fn clamp_scroll(&mut self) {
        let n = self.engine.filtered_len();
        if n == 0 {
            self.scroll_offset = 0;
            self.wrap_skip = 0;
            return;
        }
        if self.scroll_offset >= n {
            self.scroll_offset = n - 1;
            self.wrap_skip = 0;
        }
        if !self.soft_wrap {
            let max = self.max_scroll();
            if self.scroll_offset > max {
                self.scroll_offset = max;
            }
            self.wrap_skip = 0;
        } else {
            let width = self.viewport_width.max(1);
            let h = self.entry_wrap_height(self.scroll_offset, width);
            if self.wrap_skip >= h {
                self.wrap_skip = h.saturating_sub(1);
            }
        }
    }

    fn ensure_row_visible(&mut self, row: usize) {
        let n = self.engine.filtered_len();
        if row >= n {
            return;
        }
        if self.soft_wrap {
            // Bring entry into view; start at that entry
            if row < self.scroll_offset {
                self.scroll_offset = row;
                self.wrap_skip = 0;
            } else {
                // If not already visible in current window, jump so entry is near top
                let width = self.viewport_width.max(1);
                let mut lines = 0usize;
                let mut idx = self.scroll_offset;
                let mut skip = self.wrap_skip;
                let mut visible = false;
                while idx < n && lines < self.viewport_height {
                    if idx == row {
                        visible = true;
                        break;
                    }
                    let h = self.entry_wrap_height(idx, width);
                    lines += h.saturating_sub(skip);
                    skip = 0;
                    idx += 1;
                }
                if !visible {
                    self.scroll_offset = row;
                    self.wrap_skip = 0;
                }
            }
            return;
        }
        let h = self.viewport_height.max(1);
        if row < self.scroll_offset {
            self.scroll_offset = row;
        } else if row >= self.scroll_offset + h {
            self.scroll_offset = row + 1 - h;
        }
    }

    fn mark_filter_dirty(&mut self) {
        self.filter_dirty = true;
        self.filter_changed_at = Some(Instant::now());
    }

    fn cycle_level(&mut self, forward: bool) {
        let levels: [Option<LogLevel>; 6] = [
            None,
            Some(LogLevel::Debug),
            Some(LogLevel::Info),
            Some(LogLevel::Warn),
            Some(LogLevel::Error),
            Some(LogLevel::Fatal),
        ];
        let idx = levels
            .iter()
            .position(|l| *l == self.filter_level)
            .unwrap_or(0);
        let next = if forward {
            (idx + 1) % levels.len()
        } else {
            (idx + levels.len() - 1) % levels.len()
        };
        self.filter_level = levels[next];
        self.mark_filter_dirty();
    }

    fn cycle_preset(&mut self, forward: bool) {
        let was_custom_focus =
            self.settings_panel.focus_field == SettingsField::Custom;
        let all = BufferPreset::ALL;
        let idx = all
            .iter()
            .position(|p| *p == self.settings_panel.preset)
            .unwrap_or(0);
        let next = if forward {
            (idx + 1) % all.len()
        } else {
            (idx + all.len() - 1) % all.len()
        };
        self.settings_panel.preset = all[next];
        if let Some(cap) = self.settings_panel.preset.capacity() {
            self.settings_panel.custom_capacity = cap.to_string();
        }
        if was_custom_focus && self.settings_panel.preset != BufferPreset::Custom {
            // Custom sits between Preset and Language; re-anchor to Preset.
            self.settings_panel.focus_field = SettingsField::Preset;
        }
        self.commit_settings_from_panel();
    }

    fn cycle_language(&mut self, forward: bool) {
        self.settings_panel.language = self.settings_panel.language.cycle(forward);
        self.commit_settings_from_panel();
    }

    fn commit_settings_from_panel(&mut self) {
        let settings = Settings {
            adb_path: if self.settings_panel.adb_path.trim().is_empty() {
                None
            } else {
                Some(self.settings_panel.adb_path.trim().to_string())
            },
            buffer_capacity: self.settings_panel.capacity(),
            auto_scroll_to_end: self.auto_scroll,
            soft_wrap: self.soft_wrap,
            language: self.settings_panel.language,
        };
        match save_settings(&settings) {
            Ok(()) => {
                self.settings.adb_path = settings.adb_path.clone();
                self.settings.buffer_capacity = settings.buffer_capacity;
                self.settings.language = settings.language;
                self.locale = Locale::resolve(settings.language);
                self.ui = UiStrings::for_locale(self.locale);
                self.engine.set_capacity(settings.buffer_capacity);
                self.settings_panel.status = None;
            }
            Err(e) => self.settings_panel.status = Some(e),
        }
    }

    fn persist_display_prefs(&mut self) {
        self.settings.auto_scroll_to_end = self.auto_scroll;
        self.settings.soft_wrap = self.soft_wrap;
        let _ = save_settings(&self.settings);
    }

    fn apply_filter(&mut self) {
        let tag = if self.filter_tag.text.is_empty() {
            None
        } else {
            Some(self.filter_tag.text.clone())
        };
        let message = if self.filter_message.text.is_empty() {
            None
        } else {
            Some(self.filter_message.text.clone())
        };
        self.engine
            .set_filter(tag, message, self.filter_level);
        self.selection.clear();
        self.follow_dirty = true;
        if self.find.open {
            self.find.recompute(&self.engine);
        }
        self.clamp_scroll();
        self.seed_or_clamp_caret();
    }

    fn refresh_devices(&mut self) {
        self.last_device_refresh = Instant::now();
        match adb::resolve_adb_path(self.settings.adb_path.as_deref()) {
            Ok(path) => {
                if let Err(e) = adb::check_adb_version(&path) {
                    self.last_error = Some(e);
                    return;
                }
                match adb::list_devices(&path) {
                    Ok(devices) => {
                        self.devices = devices;
                        self.last_error = None;
                    }
                    Err(e) => self.last_error = Some(e),
                }
            }
            Err(e) => self.last_error = Some(e),
        }
    }

    fn start_selected_device(&mut self) {
        self.engine.stop_stream();
        let Some(serial) = self.selected_serial.clone() else {
            return;
        };
        match adb::resolve_adb_path(self.settings.adb_path.as_deref()) {
            Ok(path) => {
                if let Err(e) = adb::check_adb_version(&path) {
                    self.last_error = Some(e);
                    return;
                }
                self.engine
                    .start_stream(self.runtime_handle(), path, serial);
                self.last_error = None;
                self.scroll_offset = 0;
            }
            Err(e) => self.last_error = Some(e),
        }
    }

    fn drain_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                EngineEvent::RowsAppended(n) => {
                    self.follow_dirty = true;
                    if self.find.open {
                        self.find.append_search(&self.engine, n);
                    }
                }
                EngineEvent::DroppedFront(n) => {
                    self.follow_dirty = true;
                    if self.find.open {
                        self.find.on_dropped_front(n);
                    }
                    self.scroll_offset = self.scroll_offset.saturating_sub(n);
                    self.selection.clear();
                }
                EngineEvent::Stats(stats) => {
                    self.stats = stats;
                }
                EngineEvent::Error(err) => {
                    self.last_error = Some(err);
                }
                EngineEvent::Cleared => {
                    self.follow_dirty = true;
                    if self.find.open {
                        self.find.recompute(&self.engine);
                    }
                    self.clamp_scroll();
                }
            }
        }
    }

    // --- drawing ---

    pub fn draw(&mut self, frame: &mut Frame) {
        self.hit_map = HitMap::default();
        let area = frame.area();
        let content = shell_content_area(area);

        let find_h = if self.find.open { 1u16 } else { 0 };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // toolbar
                Constraint::Length(1), // separator
                Constraint::Length(1), // filters
                Constraint::Length(1), // separator
                Constraint::Length(find_h),
                Constraint::Min(3),    // logs
                Constraint::Length(1), // separator
                Constraint::Length(1), // status
            ])
            .split(content);

        self.draw_toolbar(frame, chunks[0]);
        self.draw_separator(frame, chunks[1]);
        self.draw_filters(frame, chunks[2]);
        self.draw_separator(frame, chunks[3]);
        if self.find.open {
            self.draw_find(frame, chunks[4]);
        }
        self.draw_logs(frame, chunks[5]);
        self.draw_separator(frame, chunks[6]);
        self.draw_status(frame, chunks[7]);

        if self.modal.is_some() {
            self.draw_modal(frame, area);
        }

        self.apply_pointer_shape();
    }

    fn draw_toolbar(&mut self, frame: &mut Frame, area: Rect) {
        let paused = self.engine.is_paused();
        let pause_label = if paused {
            self.ui.toolbar_resume
        } else {
            self.ui.toolbar_pause
        };
        let follow_mark = if self.auto_scroll { "*" } else { " " };
        let wrap_mark = if self.soft_wrap { "*" } else { " " };
        let device_label = self
            .selected_serial
            .as_deref()
            .unwrap_or(self.ui.none_device);
        let ui = self.ui;

        let labels: Vec<(String, ToolbarHit)> = vec![
            (
                format!("[d]{}:{device_label}", ui.toolbar_dev),
                ToolbarHit::Devices,
            ),
            (format!("[Space]{pause_label}"), ToolbarHit::Pause),
            (format!("[c]{}", ui.toolbar_clear), ToolbarHit::Clear),
            (
                format!("[f]{}{follow_mark}", ui.toolbar_follow),
                ToolbarHit::Follow,
            ),
            (
                format!("[w]{}{wrap_mark}", ui.toolbar_wrap),
                ToolbarHit::Wrap,
            ),
            (format!("[e]{}", ui.toolbar_export), ToolbarHit::Export),
            (
                format!("[s]{}", ui.toolbar_settings),
                ToolbarHit::Settings,
            ),
        ];

        let mut spans = Vec::new();
        let mut x = area.x;
        for (i, (label, hit)) in labels.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" │ "));
                x = x.saturating_add(3);
            }
            let width = str_display_width(label);
            let rect = Rect {
                x,
                y: area.y,
                width,
                height: 1,
            };
            self.hit_map.toolbar.push((rect, *hit));
            spans.push(Span::styled(
                label.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            ));
            x = x.saturating_add(width);
        }
        spans.push(Span::raw(" │ "));
        spans.push(Span::styled(
            format!("[q]{}", ui.toolbar_quit),
            Style::default().add_modifier(Modifier::BOLD),
        ));

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    fn draw_separator(&self, frame: &mut Frame, area: Rect) {
        let line = "─".repeat(area.width as usize);
        frame.render_widget(Paragraph::new(line), area);
    }

    fn draw_filters(&mut self, frame: &mut Frame, area: Rect) {
        let level_text = self
            .filter_level
            .map(|l| l.to_display())
            .unwrap_or("Verbose");
        let ui = self.ui;

        let summary_style = Style::default();
        let shortcut_style = Style::default().add_modifier(Modifier::BOLD);
        let level_style = field_style(self.focus == Focus::Level, &self.theme);

        let tag_value = truncate_input(&self.filter_tag.text, 16);
        let msg_value = truncate_input(&self.filter_message.text, 24);
        let tag_label = format!("[t]{}[{tag_value}] ", ui.filter_tag);
        let msg_label = format!("[m]{}[{msg_value}] ", ui.filter_message);
        let level_label = format!("[l]{}[{level_text}]", ui.filter_level);

        let mut x = area.x;
        let tag_w = str_display_width(&tag_label);
        self.hit_map.filter_tag = Some(Rect {
            x,
            y: area.y,
            width: tag_w,
            height: 1,
        });
        x = x.saturating_add(tag_w + 1);
        let msg_w = str_display_width(&msg_label);
        self.hit_map.filter_message = Some(Rect {
            x,
            y: area.y,
            width: msg_w,
            height: 1,
        });
        x = x.saturating_add(msg_w + 1);
        let level_w = str_display_width(&level_label);
        self.hit_map.filter_level = Some(Rect {
            x,
            y: area.y,
            width: level_w,
            height: 1,
        });

        let line = Line::from(vec![
            Span::styled("[t]", shortcut_style),
            Span::styled(format!("{}[{tag_value}] ", ui.filter_tag), summary_style),
            Span::styled("[m]", shortcut_style),
            Span::styled(
                format!("{}[{msg_value}] ", ui.filter_message),
                summary_style,
            ),
            Span::styled("[l]", shortcut_style),
            Span::styled(format!("{}[{level_text}]", ui.filter_level), level_style),
            Span::styled(ui.filter_click_hint, summary_style),
        ]);
        frame.render_widget(Paragraph::new(line), area);
    }

    fn draw_find(&mut self, frame: &mut Frame, area: Rect) {
        let style = field_style(self.focus == Focus::Find, &self.theme);
        let counter = self.find.counter_text(self.ui.find_zero_matches);
        let query = &self.find.input.text;
        let prefix = self.ui.find_prefix;
        let suffix = format!("] {counter}{}", self.ui.find_help_suffix);
        let value_width = str_display_width(query).max(1);
        let prefix_width = str_display_width(prefix);
        let value_start_col = area.x.saturating_add(prefix_width);
        self.hit_map.find_input = Some(Rect {
            x: value_start_col,
            y: area.y,
            width: value_width,
            height: 1,
        });

        if self.focus == Focus::Find {
            let cursor_x = value_start_col.saturating_add(self.find.input.display_width_before_cursor());
            frame.set_cursor_position((cursor_x, area.y));
        }

        let text = format!("{prefix}{query}{suffix}");
        frame.render_widget(Paragraph::new(Span::styled(text, style)), area);
    }

    fn draw_logs(&mut self, frame: &mut Frame, area: Rect) {
        self.hit_map.log_viewport = Some(area);
        let prev_h = self.viewport_height;
        let prev_w = self.viewport_width;
        self.viewport_height = area.height as usize;
        self.viewport_width = area.width as usize;
        // Viewport size is only known here; re-follow when it changes so Wrap
        // mode lands on the true end of a long line (not the default 10-row math).
        if self.viewport_height != prev_h || self.viewport_width != prev_w {
            self.follow_dirty = true;
        }
        self.apply_follow_scroll_if_needed();

        let row_count = self.engine.filtered_len();
        let find_q = if self.find.open {
            self.find.input.text.trim().to_lowercase()
        } else {
            String::new()
        };
        let current_row = if self.find.open {
            self.find.current_row()
        } else {
            None
        };

        let mut items: Vec<ListItem> = Vec::new();

        if row_count == 0 {
            items.push(ListItem::new(Span::raw(self.ui.empty_logs)));
            frame.render_widget(List::new(items), area);
            return;
        }

        if self.soft_wrap {
            let width = self.viewport_width.max(1);
            let height = self.viewport_height;
            // Fetch a window large enough to fill the viewport even if every
            // entry is a single display line.
            let fetch_end = (self.scroll_offset + height + 1).min(row_count);
            let entries = self
                .engine
                .copy_filtered_range(self.scroll_offset, fetch_end);

            let mut skip = self.wrap_skip;
            for (i, entry) in entries.iter().enumerate() {
                if items.len() >= height {
                    break;
                }
                let row_idx = self.scroll_offset + i;
                let line_str = format_log_line(entry);
                let base_color = self.theme.level_color(entry.level);
                let is_current = current_row == Some(row_idx);
                let indent = message_column_indent(entry);

                let mut skip_rows = skip;
                skip = 0;
                for (logical_start, chunk) in WrapChunks::with_indent(&line_str, width, indent) {
                    if skip_rows > 0 {
                        skip_rows -= 1;
                        continue;
                    }
                    if items.len() >= height {
                        break;
                    }
                    let display = wrap_display_text(&chunk, logical_start, indent, width);
                    let display_pad = if logical_start > 0 {
                        effective_hang_indent(indent, width)
                    } else {
                        0
                    };
                    let spans = line_spans(
                        &display,
                        row_idx,
                        logical_start,
                        display_pad,
                        base_color,
                        &self.theme,
                        &self.selection,
                        &find_q,
                        is_current && logical_start == 0,
                    );
                    items.push(ListItem::new(Line::from(spans)));
                }
            }
        } else {
            let start = self.scroll_offset.min(row_count);
            let end = (start + self.viewport_height).min(row_count);
            let entries = self.engine.copy_filtered_range(start, end);

            for (i, entry) in entries.iter().enumerate() {
                let row_idx = start + i;
                let line_str = format_log_line(entry);
                let visible = visible_chars(&line_str, self.col_offset, self.viewport_width);
                let base_color = self.theme.level_color(entry.level);
                let is_current = current_row == Some(row_idx);
                let spans = line_spans(
                    &visible,
                    row_idx,
                    self.col_offset,
                    0,
                    base_color,
                    &self.theme,
                    &self.selection,
                    &find_q,
                    is_current,
                );
                items.push(ListItem::new(Line::from(spans)));
            }
        }

        frame.render_widget(List::new(items), area);

        if !self.text_input_focused() {
            if let Some((x, y)) = self.log_caret_screen_pos() {
                frame.set_cursor_position((x, y));
            }
        }
    }

    fn draw_status(&self, frame: &mut Frame, area: Rect) {
        let live = self.engine.is_streaming();
        let live_txt = if live {
            self.ui.status_live
        } else {
            self.ui.status_idle
        };
        let focus_hint = match self.focus {
            Focus::Logs => self.ui.focus_logs,
            Focus::Level => self.ui.focus_level,
            Focus::Find => self.ui.focus_find,
            Focus::Modal => self.ui.focus_modal,
        };
        let err = self
            .last_error
            .as_deref()
            .or(self.status_message.as_deref())
            .unwrap_or("");
        let wrap_hint = if self.soft_wrap {
            self.ui.wrap_on
        } else {
            self.ui.wrap_off
        };
        let filtered = self.engine.filtered_len();
        let text = format!(
            "{}  {}/{}/{}  {:.0} lines/s  ~{:.1} MB  {}  {}  {}",
            live_txt,
            filtered,
            self.stats.count,
            self.stats.capacity,
            self.stats.lines_per_sec,
            self.stats.memory_estimate_mb,
            focus_hint,
            wrap_hint,
            err
        );
        frame.render_widget(Paragraph::new(text), area);
    }

    fn draw_modal(&mut self, frame: &mut Frame, area: Rect) {
        let kind = self.modal.clone();
        let Some(kind) = kind else { return };

        let popup = centered_rect(60, 50, area);
        frame.render_widget(Clear, popup);

        let ui = self.ui;
        match kind {
            ModalKind::Devices => {
                let mut lines = vec![
                    Line::from(ui.modal_devices_help),
                    Line::from(""),
                ];
                let none_selected = self.device_cursor == 0;
                lines.push(Line::from(format!(
                    "{} {}",
                    if none_selected { ">" } else { " " },
                    ui.none_device
                )));
                for (i, d) in self.devices.iter().enumerate() {
                    let sel = self.device_cursor == i + 1;
                    lines.push(Line::from(format!(
                        "{} {} ({})",
                        if sel { ">" } else { " " },
                        d.serial,
                        d.state
                    )));
                }
                let block = Block::default()
                    .title(ui.modal_devices_title)
                    .borders(Borders::ALL);
                frame.render_widget(Paragraph::new(lines).block(block), popup);
            }
            ModalKind::ExportMenu => {
                let lines = vec![
                    Line::from(ui.toolbar_export),
                    Line::from(""),
                    Line::from(ui.modal_export_filtered),
                    Line::from(ui.modal_export_all),
                    Line::from(""),
                    Line::from(ui.modal_export_cancel),
                ];
                let block = Block::default()
                    .title(ui.modal_export_title)
                    .borders(Borders::ALL);
                frame.render_widget(Paragraph::new(lines).block(block), popup);
            }
            ModalKind::Export {
                filtered_only,
                path,
            } => {
                let title = if filtered_only {
                    ui.modal_export_filtered_title
                } else {
                    ui.modal_export_all_title
                };
                let lines = vec![
                    Line::from(ui.modal_export_path_prompt),
                    Line::from(""),
                    Line::from(format!("[{}]", path)),
                ];
                let block = Block::default().title(title).borders(Borders::ALL);
                frame.render_widget(Paragraph::new(lines).block(block), popup);
            }
            ModalKind::Settings => {
                let focus = self.settings_panel.focus_field;
                let mark = |field: SettingsField| if focus == field { ">" } else { " " };
                let mut lines = vec![
                    Line::from(ui.modal_settings_help),
                    Line::from(""),
                ];
                for &field in SettingsPanelState::visible_fields(self.settings_panel.preset) {
                    match field {
                        SettingsField::Adb => {
                            let mode = if self.settings_panel.adb_is_custom() {
                                ui.settings_adb_custom
                            } else {
                                ui.settings_adb_auto
                            };
                            if self.settings_panel.adb_editing {
                                lines.push(Line::from(format!(
                                    "{} {}: [{}]",
                                    mark(field),
                                    ui.settings_adb,
                                    self.settings_panel.adb_path
                                )));
                            } else {
                                let path = self.settings_panel.adb_display_path();
                                let path_display = if path.is_empty() {
                                    ui.settings_adb_not_found.trim().to_string()
                                } else {
                                    path
                                };
                                lines.push(Line::from(format!(
                                    "{} {}: {} [{}]",
                                    mark(field),
                                    ui.settings_adb,
                                    path_display,
                                    mode
                                )));
                                if focus == SettingsField::Adb {
                                    lines.push(Line::from(ui.settings_adb_locked_hint));
                                }
                            }
                        }
                        SettingsField::Preset => lines.push(Line::from(format!(
                            "{} {}: {}",
                            mark(field),
                            ui.settings_preset,
                            self.settings_panel.preset.label()
                        ))),
                        SettingsField::Custom => lines.push(Line::from(format!(
                            "{} {}: [{}]",
                            mark(field),
                            ui.settings_custom,
                            self.settings_panel.custom_capacity
                        ))),
                        SettingsField::Language => lines.push(Line::from(format!(
                            "{} {}: {}",
                            mark(field),
                            ui.settings_language,
                            self.settings_panel.language.label()
                        ))),
                    }
                }
                if let Some(ref s) = self.settings_panel.status {
                    lines.push(Line::from(""));
                    lines.push(Line::from(s.clone()));
                }
                let block = Block::default()
                    .title(ui.modal_settings_title)
                    .borders(Borders::ALL);
                frame.render_widget(Paragraph::new(lines).block(block), popup);
            }
            ModalKind::FilterEdit { field } => {
                let (title, label, input) = match field {
                    FilterField::Tag => (
                        ui.modal_tag_filter_title,
                        ui.filter_tag_contains,
                        &self.filter_tag,
                    ),
                    FilterField::Message => (
                        ui.modal_message_filter_title,
                        ui.filter_message_contains,
                        &self.filter_message,
                    ),
                };
                let value = &input.text;
                let prefix = format!("{label} [");
                let prefix_width = str_display_width(&prefix);
                let block = Block::default().title(title).borders(Borders::ALL);
                let inner = block.inner(popup);
                let value_start_col = inner.x.saturating_add(prefix_width);
                let value_width = str_display_width(value).max(1);
                self.hit_map.filter_modal_input = Some(Rect {
                    x: value_start_col,
                    y: inner.y,
                    width: value_width,
                    height: 1,
                });

                if self.focus == Focus::Modal {
                    let cursor_x =
                        value_start_col.saturating_add(input.display_width_before_cursor());
                    frame.set_cursor_position((cursor_x, inner.y));
                }

                let lines = vec![
                    Line::from(format!("{label} [{value}]")),
                    Line::from(""),
                    Line::from(ui.filter_live_hint),
                ];
                frame.render_widget(Paragraph::new(lines).block(block), popup);
            }
        }
    }
}

fn is_copy_shortcut(key: &KeyEvent) -> bool {
    if !matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C')) {
        return false;
    }
    let m = key.modifiers;
    // macOS Command (SUPER) / legacy META
    if m.contains(KeyModifiers::SUPER) || m.contains(KeyModifiers::META) {
        return true;
    }
    // Ctrl+C fallback when the terminal does not report Command
    m.contains(KeyModifiers::CONTROL)
}

fn contains(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x
        && col < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

fn field_style(focused: bool, theme: &Theme) -> Style {
    if focused {
        Style::default()
            .fg(theme.focus_fg)
            .bg(theme.focus_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}

fn truncate_input(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{t}…")
    }
}

fn shell_content_area(area: Rect) -> Rect {
    let (pad_x, pad_y) = if area.width >= 60 && area.height >= 15 {
        (1, 1)
    } else {
        (0, 0)
    };

    if pad_x == 0 && pad_y == 0 {
        return area;
    }

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(pad_y),
            Constraint::Min(0),
            Constraint::Length(pad_y),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(pad_x),
            Constraint::Min(0),
            Constraint::Length(pad_x),
        ])
        .split(vertical[1])[1]
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}
