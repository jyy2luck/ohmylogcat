//! Terminal UI application: layout, focus, and controls over `Engine`.

use crate::adb::{self, Device};
use crate::engine::{Engine, EngineEvent};
use crate::parser::LogLevel;
use crate::settings::{
    load_settings, save_settings, BufferPreset, BufferStats, Settings,
};
use crate::ui::{format_log_line, line_spans, mouse_to_log_pos, reset_pointer_shape, set_pointer_shape, visible_chars, wrap_line_count, FindState, LogPos, PointerShape, TextInput, TextSelection, Theme, ThemePreference, ViewportMap, WrapChunks, TEXT_INPUT_CURSOR_STYLE};
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
    Theme,
}

#[derive(Debug)]
pub struct SettingsPanelState {
    pub adb_path: String,
    pub preset: BufferPreset,
    pub custom_capacity: String,
    pub theme: ThemePreference,
    pub status: Option<String>,
    focus_field: SettingsField,
}

impl SettingsPanelState {
    pub fn from_settings(settings: &Settings) -> Self {
        let preset = BufferPreset::from_capacity(settings.buffer_capacity);
        Self {
            adb_path: settings.adb_path.clone().unwrap_or_default(),
            preset,
            custom_capacity: settings.buffer_capacity.to_string(),
            theme: settings.theme,
            status: None,
            focus_field: SettingsField::Adb,
        }
    }

    fn visible_fields(preset: BufferPreset) -> &'static [SettingsField] {
        static WITH_CUSTOM: [SettingsField; 4] = [
            SettingsField::Adb,
            SettingsField::Preset,
            SettingsField::Custom,
            SettingsField::Theme,
        ];
        static WITHOUT_CUSTOM: [SettingsField; 3] = [
            SettingsField::Adb,
            SettingsField::Preset,
            SettingsField::Theme,
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
        self.focus_field = fields[next];
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
    devices: Vec<Device>,
    selected_serial: Option<String>,
    device_cursor: usize,

    filter_tag: TextInput,
    filter_message: TextInput,
    filter_level: Option<LogLevel>,

    stats: BufferStats,
    last_error: Option<String>,
    status_message: Option<String>,

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
    last_mouse: Option<(u16, u16)>,
    last_pointer: Option<PointerShape>,
    last_text_input_focused: Option<bool>,
    follow_dirty: bool,
    should_quit: bool,
}

impl OhmylogcatApp {
    pub fn new() -> Self {
        let rt = Runtime::new().expect("tokio runtime");
        let settings = load_settings();
        let theme = Theme::resolve(settings.theme);
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
            last_mouse: None,
            last_pointer: None,
            last_text_input_focused: None,
            follow_dirty: true,
            should_quit: false,
            settings,
            theme,
        };
        app.refresh_devices();
        app
    }

    fn runtime_handle(&self) -> tokio::runtime::Handle {
        self._rt.handle().clone()
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn tick(&mut self) {
        self.drain_events();

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
            KeyCode::Up | KeyCode::Char('k') => self.scroll_by(-1),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_by(1),
            KeyCode::PageUp => self.scroll_by(-(self.viewport_height as isize)),
            KeyCode::PageDown => self.scroll_by(self.viewport_height as isize),
            KeyCode::Home => {
                self.scroll_offset = 0;
                self.disable_follow_if_needed();
            }
            KeyCode::End => {
                self.scroll_to_bottom();
                if !self.auto_scroll {
                    self.auto_scroll = true;
                    self.persist_display_prefs();
                }
            }
            KeyCode::Left | KeyCode::Char('h') if !self.soft_wrap => {
                self.col_offset = self.col_offset.saturating_sub(4);
            }
            KeyCode::Right if !self.soft_wrap => {
                self.col_offset = self.col_offset.saturating_add(4);
            }
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
            KeyCode::Esc => self.close_modal(),
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
                        self.status_message = Some(format!("Exported to {}", path));
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
            KeyCode::Esc => self.close_modal(),
            KeyCode::Enter => self.save_settings_panel(),
            KeyCode::Up | KeyCode::Char('k') => self.settings_panel.move_focus(-1),
            KeyCode::Down | KeyCode::Char('j') => self.settings_panel.move_focus(1),
            KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l')
                if self.settings_panel.focus_field == SettingsField::Preset =>
            {
                let forward = matches!(key.code, KeyCode::Right | KeyCode::Char('l'));
                self.cycle_preset(forward);
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l')
                if self.settings_panel.focus_field == SettingsField::Theme =>
            {
                let forward = matches!(key.code, KeyCode::Right | KeyCode::Char('l'));
                self.cycle_theme(forward);
            }
            KeyCode::Backspace => {
                match self.settings_panel.focus_field {
                    SettingsField::Adb => {
                        self.settings_panel.adb_path.pop();
                    }
                    SettingsField::Custom => {
                        self.settings_panel.custom_capacity.pop();
                    }
                    _ => {}
                }
            }
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !matches!(
                        self.settings_panel.focus_field,
                        SettingsField::Preset | SettingsField::Theme
                    ) =>
            {
                match self.settings_panel.focus_field {
                    SettingsField::Adb => self.settings_panel.adb_path.push(c),
                    SettingsField::Custom => {
                        if c.is_ascii_digit() {
                            self.settings_panel.custom_capacity.push(c);
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
                    } else if self.hit_map.log_viewport.is_some_and(|r| contains(r, col, row)) {
                        self.selection.start(pos);
                        self.focus = Focus::Logs;
                    }
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                let was_dragging = self.selection.dragging();
                self.selection.finish_drag();
                if was_dragging && self.selection.has_extent() {
                    self.copy_selection();
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
        if let Some(pos) = self.mouse_to_log_pos(col, row) {
            self.selection.start(pos);
            self.focus = Focus::Logs;
            true
        } else {
            false
        }
    }

    fn mouse_to_log_pos(&self, col: u16, row: u16) -> Option<LogPos> {
        let area = self.hit_map.log_viewport?;
        let row_count = self.engine.filtered_len();
        let map = ViewportMap {
            area,
            scroll_offset: self.scroll_offset,
            wrap_skip: self.wrap_skip,
            col_offset: self.col_offset,
            soft_wrap: self.soft_wrap,
            viewport_width: self.viewport_width,
            viewport_height: self.viewport_height,
        };
        mouse_to_log_pos(col, row, &map, |idx| {
            self.engine
                .filtered_get(idx)
                .map(|e| format_log_line(&e))
        }, row_count)
    }

    fn copy_selection(&mut self) -> bool {
        if !self.selection.is_active() {
            return false;
        }
        let Some(text) = self.selection.extract_text(|idx| {
            self.engine
                .filtered_get(idx)
                .map(|e| format_log_line(&e))
        }) else {
            return false;
        };
        match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(text)) {
            Ok(()) => {
                self.status_message = Some("Copied selection".into());
                true
            }
            Err(e) => {
                self.last_error = Some(format!("Copy failed: {e}"));
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
        let focused = self.text_input_focused();
        if self.last_text_input_focused == Some(focused) {
            return;
        }
        self.last_text_input_focused = Some(focused);
        let style = if focused {
            TEXT_INPUT_CURSOR_STYLE
        } else {
            SetCursorStyle::DefaultUserShape
        };
        let _ = execute!(io::stdout(), style);
    }

    fn text_input_focused(&self) -> bool {
        match self.focus {
            Focus::Find if self.find.open => true,
            Focus::Modal => matches!(self.modal, Some(ModalKind::FilterEdit { .. })),
            _ => false,
        }
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
        self.selection.clear();
        if self.find.open {
            self.find.recompute(&self.engine);
        }
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
            Some(e) => wrap_line_count(&format_log_line(&e), width),
            None => 1,
        }
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
            self.settings_panel.focus_field = SettingsField::Preset;
        }
    }

    fn cycle_theme(&mut self, forward: bool) {
        self.settings_panel.theme = self.settings_panel.theme.cycle(forward);
    }

    fn save_settings_panel(&mut self) {
        let settings = Settings {
            adb_path: if self.settings_panel.adb_path.trim().is_empty() {
                None
            } else {
                Some(self.settings_panel.adb_path.trim().to_string())
            },
            buffer_capacity: self.settings_panel.capacity(),
            auto_scroll_to_end: self.auto_scroll,
            soft_wrap: self.soft_wrap,
            theme: self.settings_panel.theme,
        };
        match save_settings(&settings) {
            Ok(()) => {
                self.settings.adb_path = settings.adb_path.clone();
                self.settings.buffer_capacity = settings.buffer_capacity;
                self.settings.theme = settings.theme;
                self.theme = Theme::resolve(settings.theme);
                self.engine.set_capacity(settings.buffer_capacity);
                self.status_message = Some("Settings saved".into());
                self.close_modal();
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
    }

    fn refresh_devices(&mut self) {
        self.last_device_refresh = Instant::now();
        match adb::resolve_adb_path(self.settings.adb_path.as_deref()) {
            Ok(path) => match adb::list_devices(&path) {
                Ok(devices) => {
                    self.devices = devices;
                    self.last_error = None;
                }
                Err(e) => self.last_error = Some(e),
            },
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
            .split(area);

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
        let pause_label = if paused { "Resume" } else { "Pause" };
        let follow_mark = if self.auto_scroll { "*" } else { " " };
        let wrap_mark = if self.soft_wrap { "*" } else { " " };
        let device_label = self
            .selected_serial
            .as_deref()
            .unwrap_or("(none)");

        let labels: Vec<(String, ToolbarHit)> = vec![
            (format!("[d]Dev:{device_label}"), ToolbarHit::Devices),
            (format!("[Space]{pause_label}"), ToolbarHit::Pause),
            ("[c]Clear".into(), ToolbarHit::Clear),
            (format!("[f]Follow{follow_mark}"), ToolbarHit::Follow),
            (format!("[w]Wrap{wrap_mark}"), ToolbarHit::Wrap),
            ("[e]Export".into(), ToolbarHit::Export),
            ("[s]Settings".into(), ToolbarHit::Settings),
        ];

        let mut spans = Vec::new();
        let mut x = area.x;
        for (i, (label, hit)) in labels.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" │ "));
                x = x.saturating_add(3);
            }
            let width = label.chars().count() as u16;
            let rect = Rect {
                x,
                y: area.y,
                width,
                height: 1,
            };
            self.hit_map.toolbar.push((rect, *hit));
            spans.push(Span::styled(
                label.clone(),
                Style::default()
                    .fg(self.theme.shell_fg)
                    .add_modifier(Modifier::BOLD),
            ));
            x = x.saturating_add(width);
        }
        spans.push(Span::raw(" │ "));
        spans.push(Span::styled(
            "[q]Quit",
            Style::default()
                .fg(self.theme.shell_fg)
                .add_modifier(Modifier::BOLD),
        ));

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    fn draw_separator(&self, frame: &mut Frame, area: Rect) {
        let line = "─".repeat(area.width as usize);
        frame.render_widget(
            Paragraph::new(line).style(Style::default().fg(self.theme.shell_divider)),
            area,
        );
    }

    fn draw_filters(&mut self, frame: &mut Frame, area: Rect) {
        let level_text = self
            .filter_level
            .map(|l| l.to_display())
            .unwrap_or("Verbose");

        let summary_style = Style::default().fg(self.theme.shell_fg);
        let shortcut_style = Style::default()
            .fg(self.theme.shell_fg)
            .add_modifier(Modifier::BOLD);
        let level_style = field_style(self.focus == Focus::Level, &self.theme);

        let tag_value = truncate_input(&self.filter_tag.text, 16);
        let msg_value = truncate_input(&self.filter_message.text, 24);
        let tag_label = format!("[t]Tag[{tag_value}] ");
        let msg_label = format!("[m]Message[{msg_value}] ");
        let level_label = format!("[l]Level[{level_text}]");

        let mut x = area.x;
        let tag_w = tag_label.chars().count() as u16;
        self.hit_map.filter_tag = Some(Rect {
            x,
            y: area.y,
            width: tag_w,
            height: 1,
        });
        x = x.saturating_add(tag_w + 1);
        let msg_w = msg_label.chars().count() as u16;
        self.hit_map.filter_message = Some(Rect {
            x,
            y: area.y,
            width: msg_w,
            height: 1,
        });
        x = x.saturating_add(msg_w + 1);
        let level_w = level_label.chars().count() as u16;
        self.hit_map.filter_level = Some(Rect {
            x,
            y: area.y,
            width: level_w,
            height: 1,
        });

        let line = Line::from(vec![
            Span::styled("[t]", shortcut_style),
            Span::styled(format!("Tag[{tag_value}] "), summary_style),
            Span::styled("[m]", shortcut_style),
            Span::styled(format!("Message[{msg_value}] "), summary_style),
            Span::styled("[l]", shortcut_style),
            Span::styled(format!("Level[{level_text}]"), level_style),
            Span::raw("  (click Tag/Message)"),
        ]);
        frame.render_widget(Paragraph::new(line), area);
    }

    fn draw_find(&mut self, frame: &mut Frame, area: Rect) {
        let style = field_style(self.focus == Focus::Find, &self.theme);
        let counter = self.find.counter_text();
        let query = &self.find.input.text;
        let prefix = "Find:[";
        let suffix = format!("] {counter}  (Enter next · Shift+Enter prev · Esc close)");
        let value_width = query.chars().count().max(1) as u16;
        let prefix_width = prefix.chars().count() as u16;
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
            items.push(ListItem::new(Span::styled(
                "No logs — press [d] to select a device",
                Style::default().fg(self.theme.shell_hint),
            )));
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

                // Jump directly to the first visible wrap chunk instead of
                // iterating and discarding skipped chunks one-by-one.
                let start_chunk = skip;
                let remainder = visible_chars(&line_str, start_chunk * width, usize::MAX);
                skip = 0;
                for (ci, chunk) in WrapChunks::new(&remainder, width).enumerate() {
                    if items.len() >= height {
                        break;
                    }
                    let abs_ci = start_chunk + ci;
                    let line_char_start = abs_ci * width;
                    let spans = line_spans(
                        &chunk,
                        row_idx,
                        line_char_start,
                        base_color,
                        &self.theme,
                        &self.selection,
                        &find_q,
                        is_current && abs_ci == 0,
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
    }

    fn draw_status(&self, frame: &mut Frame, area: Rect) {
        let live = self.engine.is_streaming();
        let live_txt = if live { "● Live" } else { "○ Idle" };
        let focus_hint = match self.focus {
            Focus::Logs => "focus:logs",
            Focus::Level => "focus:level",
            Focus::Find => "focus:find",
            Focus::Modal => "focus:modal",
        };
        let err = self
            .last_error
            .as_deref()
            .or(self.status_message.as_deref())
            .unwrap_or("");
        let wrap_hint = if self.soft_wrap { "wrap:on" } else { "wrap:off" };
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
        frame.render_widget(
            Paragraph::new(text).style(Style::default().fg(self.theme.shell_muted)),
            area,
        );
    }

    fn draw_modal(&mut self, frame: &mut Frame, area: Rect) {
        let kind = self.modal.clone();
        let Some(kind) = kind else { return };

        let popup = centered_rect(60, 50, area);
        frame.render_widget(Clear, popup);

        match kind {
            ModalKind::Devices => {
                let mut lines = vec![
                    Line::from("Select device  (↑↓ · Enter · r refresh · Esc)"),
                    Line::from(""),
                ];
                let none_selected = self.device_cursor == 0;
                lines.push(Line::from(format!(
                    "{} (none)",
                    if none_selected { ">" } else { " " }
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
                    .title(" Devices ")
                    .borders(Borders::ALL);
                frame.render_widget(Paragraph::new(lines).block(block), popup);
            }
            ModalKind::ExportMenu => {
                let lines = vec![
                    Line::from("Export"),
                    Line::from(""),
                    Line::from("[1]/f] Export filtered"),
                    Line::from("[2]/a] Export all"),
                    Line::from(""),
                    Line::from("Esc cancel"),
                ];
                let block = Block::default()
                    .title(" Export ")
                    .borders(Borders::ALL);
                frame.render_widget(Paragraph::new(lines).block(block), popup);
            }
            ModalKind::Export {
                filtered_only,
                path,
            } => {
                let title = if filtered_only {
                    " Export filtered "
                } else {
                    " Export all "
                };
                let lines = vec![
                    Line::from("Path (Enter confirm · Esc cancel):"),
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
                    Line::from(
                        "↑/↓ move (j/k) · ←/→ adjust (h/l) · type text · Enter save · Esc cancel",
                    ),
                    Line::from(""),
                ];
                for &field in SettingsPanelState::visible_fields(self.settings_panel.preset) {
                    lines.push(match field {
                        SettingsField::Adb => Line::from(format!(
                            "{} ADB: [{}]",
                            mark(field),
                            self.settings_panel.adb_path
                        )),
                        SettingsField::Preset => Line::from(format!(
                            "{} Preset: {}",
                            mark(field),
                            self.settings_panel.preset.label()
                        )),
                        SettingsField::Custom => Line::from(format!(
                            "{} Custom: [{}]",
                            mark(field),
                            self.settings_panel.custom_capacity
                        )),
                        SettingsField::Theme => Line::from(format!(
                            "{} Theme: {}",
                            mark(field),
                            self.settings_panel.theme.label()
                        )),
                    });
                }
                if let Some(ref s) = self.settings_panel.status {
                    lines.push(Line::from(""));
                    lines.push(Line::from(s.clone()));
                }
                let block = Block::default()
                    .title(" Settings ")
                    .borders(Borders::ALL);
                frame.render_widget(Paragraph::new(lines).block(block), popup);
            }
            ModalKind::FilterEdit { field } => {
                let (title, label, input) = match field {
                    FilterField::Tag => (
                        " Tag filter ",
                        "Tag contains:",
                        &self.filter_tag,
                    ),
                    FilterField::Message => (
                        " Message filter ",
                        "Message contains:",
                        &self.filter_message,
                    ),
                };
                let value = &input.text;
                let prefix = format!("{label} [");
                let prefix_width = prefix.chars().count() as u16;
                let block = Block::default().title(title).borders(Borders::ALL);
                let inner = block.inner(popup);
                let value_start_col = inner.x.saturating_add(prefix_width);
                let value_width = value.chars().count().max(1) as u16;
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
                    Line::from("Live filter · Esc done"),
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
        Style::default().fg(theme.shell_fg)
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
