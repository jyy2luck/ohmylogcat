//! Terminal UI application: layout, focus, and controls over `Engine`.

use crate::adb::{self, Device};
use crate::engine::{Engine, EngineEvent};
use crate::parser::LogLevel;
use crate::settings::{
    load_settings, save_settings, BufferPreset, BufferStats, Settings,
};
use crate::ui::{format_log_line, level_color, line_spans, mouse_to_log_pos, reset_pointer_shape, set_pointer_shape, FindState, LogPos, PointerShape, TextSelection, ViewportMap};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
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

#[derive(Debug)]
pub struct SettingsPanelState {
    pub adb_path: String,
    pub preset: BufferPreset,
    pub custom_capacity: String,
    pub status: Option<String>,
    pub focus_field: usize, // 0 = adb, 1 = preset, 2 = custom
}

impl SettingsPanelState {
    pub fn from_settings(settings: &Settings) -> Self {
        let preset = BufferPreset::from_capacity(settings.buffer_capacity);
        Self {
            adb_path: settings.adb_path.clone().unwrap_or_default(),
            preset,
            custom_capacity: settings.buffer_capacity.to_string(),
            status: None,
            focus_field: 0,
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
    devices: Vec<Device>,
    selected_serial: Option<String>,
    device_cursor: usize,

    filter_tag: String,
    filter_message: String,
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
    should_quit: bool,
}

impl OhmylogcatApp {
    pub fn new() -> Self {
        let rt = Runtime::new().expect("tokio runtime");
        let settings = load_settings();
        let (engine, event_rx) = Engine::new(settings.buffer_capacity);

        let mut app = Self {
            _rt: rt,
            engine,
            event_rx,
            devices: Vec::new(),
            selected_serial: None,
            device_cursor: 0,
            filter_tag: String::new(),
            filter_message: String::new(),
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
            should_quit: false,
            settings,
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
        if self.auto_scroll && !self.find.is_active_with_matches() {
            self.scroll_to_bottom();
        }
        if let Some(row) = self.find.scroll_to_match.take() {
            self.ensure_row_visible(row);
        }
    }

    pub fn handle_event(&mut self, event: Event) -> io::Result<()> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat => {
                self.handle_key(key);
            }
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            Event::Resize(_, _) => {}
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
            KeyCode::Backspace => {
                self.find.query.pop();
                self.find.recompute(&self.engine);
            }
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::SUPER) =>
            {
                self.find.query.push(c);
                self.find.recompute(&self.engine);
            }
            _ => {}
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
            KeyCode::Backspace => {
                match field {
                    FilterField::Tag => {
                        self.filter_tag.pop();
                    }
                    FilterField::Message => {
                        self.filter_message.pop();
                    }
                }
                self.mark_filter_dirty();
            }
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::SUPER) =>
            {
                match field {
                    FilterField::Tag => self.filter_tag.push(c),
                    FilterField::Message => self.filter_message.push(c),
                }
                self.mark_filter_dirty();
            }
            _ => {}
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
            KeyCode::Tab => {
                self.settings_panel.focus_field =
                    (self.settings_panel.focus_field + 1) % 3;
            }
            KeyCode::BackTab => {
                self.settings_panel.focus_field =
                    (self.settings_panel.focus_field + 2) % 3;
            }
            KeyCode::Enter => self.save_settings_panel(),
            KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l')
                if self.settings_panel.focus_field == 1 =>
            {
                let forward = matches!(key.code, KeyCode::Right | KeyCode::Char('l'));
                self.cycle_preset(forward);
            }
            KeyCode::Char('[') if self.settings_panel.focus_field == 1 => {
                self.cycle_preset(false);
            }
            KeyCode::Char(']') if self.settings_panel.focus_field == 1 => {
                self.cycle_preset(true);
            }
            KeyCode::Backspace => {
                match self.settings_panel.focus_field {
                    0 => {
                        self.settings_panel.adb_path.pop();
                    }
                    2 => {
                        self.settings_panel.custom_capacity.pop();
                    }
                    _ => {}
                }
            }
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && self.settings_panel.focus_field != 1 =>
            {
                match self.settings_panel.focus_field {
                    0 => self.settings_panel.adb_path.push(c),
                    2 => {
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
        let shape = if self.modal.is_some() {
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
            self.scroll_to_bottom();
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
        let mut lines = 0usize;
        let mut idx = self.scroll_offset;
        let mut skip = self.wrap_skip;
        while idx < n && lines < self.viewport_height {
            let h = self.entry_wrap_height(idx, width);
            lines += h.saturating_sub(skip);
            skip = 0;
            idx += 1;
        }
        // At bottom when we've consumed all entries and still have room,
        // or exactly filled through the last entry.
        idx >= n
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
        let mut lines = 0usize;
        let mut start = n;
        while start > 0 && lines < self.viewport_height {
            start -= 1;
            lines += self.entry_wrap_height(start, width);
        }
        self.scroll_offset = start;
        self.wrap_skip = lines.saturating_sub(self.viewport_height);
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
        };
        match save_settings(&settings) {
            Ok(()) => {
                self.settings.adb_path = settings.adb_path.clone();
                self.settings.buffer_capacity = settings.buffer_capacity;
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
        let tag = if self.filter_tag.is_empty() {
            None
        } else {
            Some(self.filter_tag.clone())
        };
        let message = if self.filter_message.is_empty() {
            None
        } else {
            Some(self.filter_message.clone())
        };
        self.engine
            .set_filter(tag, message, self.filter_level);
        self.selection.clear();
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
                    if self.find.open {
                        self.find.append_search(&self.engine, n);
                    }
                }
                EngineEvent::DroppedFront(n) => {
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
                Style::default().add_modifier(Modifier::BOLD),
            ));
            x = x.saturating_add(width);
        }
        spans.push(Span::raw(" │ "));
        spans.push(Span::styled(
            "[q]Quit",
            Style::default().add_modifier(Modifier::BOLD),
        ));

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    fn draw_separator(&self, frame: &mut Frame, area: Rect) {
        let line = "─".repeat(area.width as usize);
        frame.render_widget(
            Paragraph::new(line).style(Style::default().fg(Color::DarkGray)),
            area,
        );
    }

    fn draw_filters(&mut self, frame: &mut Frame, area: Rect) {
        let level_text = self
            .filter_level
            .map(|l| l.to_display())
            .unwrap_or("Verbose");

        let summary_style = Style::default();
        let shortcut_style = Style::default().add_modifier(Modifier::BOLD);
        let level_style = field_style(self.focus == Focus::Level);

        let tag_value = truncate_input(&self.filter_tag, 16);
        let msg_value = truncate_input(&self.filter_message, 24);
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
        let style = field_style(self.focus == Focus::Find);
        let counter = self.find.counter_text();
        let text = format!(
            "Find:[{}] {}  (Enter next · Shift+Enter prev · Esc close)",
            self.find.query, counter
        );
        frame.render_widget(Paragraph::new(Span::styled(text, style)), area);
    }

    fn draw_logs(&mut self, frame: &mut Frame, area: Rect) {
        self.hit_map.log_viewport = Some(area);
        self.viewport_height = area.height as usize;
        self.viewport_width = area.width as usize;

        let row_count = self.engine.filtered_len();
        let find_q = if self.find.open {
            self.find.query.trim().to_lowercase()
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
                Style::default().fg(ratatui::style::Color::DarkGray),
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
                let chunks = wrap_chunks(&line_str, width);
                let base_color = level_color(entry.level);
                let is_current = current_row == Some(row_idx);

                for (ci, chunk) in chunks.into_iter().enumerate() {
                    if skip > 0 {
                        skip -= 1;
                        continue;
                    }
                    if items.len() >= height {
                        break;
                    }
                    let line_char_start = ci * width;
                    let spans = line_spans(
                        &chunk,
                        row_idx,
                        line_char_start,
                        base_color,
                        &self.selection,
                        &find_q,
                        is_current && ci == 0,
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
                let sliced = skip_chars(&line_str, self.col_offset);
                let visible = truncate_chars(&sliced, self.viewport_width);
                let base_color = level_color(entry.level);
                let is_current = current_row == Some(row_idx);
                let spans = line_spans(
                    &visible,
                    row_idx,
                    self.col_offset,
                    base_color,
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
        let text = format!(
            "{}  {}/{}  {:.0} lines/s  ~{:.1} MB  {}  {}  {}",
            live_txt,
            self.stats.count,
            self.stats.capacity,
            self.stats.lines_per_sec,
            self.stats.memory_estimate_mb,
            focus_hint,
            wrap_hint,
            err
        );
        frame.render_widget(
            Paragraph::new(text).style(Style::default().fg(ratatui::style::Color::Gray)),
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
                let adb_mark = if self.settings_panel.focus_field == 0 {
                    ">"
                } else {
                    " "
                };
                let preset_mark = if self.settings_panel.focus_field == 1 {
                    ">"
                } else {
                    " "
                };
                let custom_mark = if self.settings_panel.focus_field == 2 {
                    ">"
                } else {
                    " "
                };
                let mut lines = vec![
                    Line::from("Tab fields · [ ] cycle preset · Enter save · Esc cancel"),
                    Line::from(""),
                    Line::from(format!(
                        "{} ADB: [{}]",
                        adb_mark, self.settings_panel.adb_path
                    )),
                    Line::from(format!(
                        "{} Preset: {}",
                        preset_mark,
                        self.settings_panel.preset.label()
                    )),
                ];
                if self.settings_panel.preset == BufferPreset::Custom {
                    lines.push(Line::from(format!(
                        "{} Custom: [{}]",
                        custom_mark, self.settings_panel.custom_capacity
                    )));
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
                let (title, label, value) = match field {
                    FilterField::Tag => (
                        " Tag filter ",
                        "Tag contains:",
                        self.filter_tag.as_str(),
                    ),
                    FilterField::Message => (
                        " Message filter ",
                        "Message contains:",
                        self.filter_message.as_str(),
                    ),
                };
                let lines = vec![
                    Line::from(format!("{label} [{value}]")),
                    Line::from(""),
                    Line::from("Live filter · Esc done"),
                ];
                let block = Block::default().title(title).borders(Borders::ALL);
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

fn field_style(focused: bool) -> Style {
    if focused {
        Style::default()
            .fg(ratatui::style::Color::Black)
            .bg(ratatui::style::Color::Yellow)
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

fn skip_chars(s: &str, n: usize) -> String {
    s.chars().skip(n).collect()
}

fn truncate_chars(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect()
    }
}

/// Number of terminal rows needed to show `s` at `width` columns.
fn wrap_line_count(s: &str, width: usize) -> usize {
    let width = width.max(1);
    let chars = s.chars().count();
    if chars == 0 {
        1
    } else {
        chars.div_ceil(width)
    }
}

/// Split `s` into chunks of at most `width` characters (Unicode scalar values).
fn wrap_chunks(s: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let chars: Vec<char> = s.chars().collect();
    if chars.is_empty() {
        return vec![String::new()];
    }
    chars
        .chunks(width)
        .map(|chunk| chunk.iter().collect())
        .collect()
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
