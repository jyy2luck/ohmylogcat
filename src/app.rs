use crate::adb::{self, Device};
use crate::engine::{Engine, EngineEvent};
use crate::parser::LogLevel;
use crate::settings::{load_settings, save_settings, BufferStats, Settings};
use crate::ui::{
    self, show_filter_bar, show_find_bar, show_log_list, show_settings_panel, show_status_bar,
    show_toolbar, FindState, SettingsPanelState, ToolbarAction,
};
use egui::{Context, TextStyle};
use std::collections::VecDeque;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

pub struct OhmylogcatApp {
    _rt: Runtime,
    engine: Arc<Engine>,
    event_rx: Receiver<EngineEvent>,

    settings: Settings,
    devices: Vec<Device>,
    selected_serial: Option<String>,
    filter_tag: String,
    filter_message: String,
    filter_level: Option<LogLevel>,

    /// Per-row height cache for Soft-Wrap virtualization (not entry bodies).
    row_heights: VecDeque<f32>,
    last_wrap_width: f32,
    stats: BufferStats,
    last_error: Option<String>,

    auto_scroll: bool,
    soft_wrap: bool,
    force_scroll_end: bool,
    prev_scroll_offset: f32,

    find: FindState,
    show_settings: bool,
    settings_panel: SettingsPanelState,

    last_device_refresh: Instant,
    status_message: Option<String>,

    /// Debounce full-buffer refilter while typing in filter fields.
    filter_dirty: bool,
    filter_changed_at: Option<Instant>,
}

impl OhmylogcatApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let rt = Runtime::new().expect("tokio runtime");
        let settings = load_settings();
        let (engine, event_rx) = Engine::new(settings.buffer_capacity);

        configure_style(&cc.egui_ctx);

        let mut app = Self {
            _rt: rt,
            engine,
            event_rx,
            devices: Vec::new(),
            selected_serial: None,
            filter_tag: String::new(),
            filter_message: String::new(),
            filter_level: None,
            row_heights: VecDeque::new(),
            last_wrap_width: 0.0,
            stats: BufferStats {
                capacity: settings.buffer_capacity,
                ..Default::default()
            },
            last_error: None,
            auto_scroll: settings.auto_scroll_to_end,
            soft_wrap: settings.soft_wrap,
            force_scroll_end: false,
            prev_scroll_offset: 0.0,
            find: FindState::default(),
            show_settings: false,
            settings_panel: SettingsPanelState::from_settings(&settings),
            last_device_refresh: Instant::now() - Duration::from_secs(60),
            status_message: None,
            settings,
            filter_dirty: false,
            filter_changed_at: None,
        };
        app.refresh_devices();
        app
    }

    fn runtime_handle(&self) -> tokio::runtime::Handle {
        self._rt.handle().clone()
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

    fn persist_display_prefs(&mut self) {
        self.settings.auto_scroll_to_end = self.auto_scroll;
        self.settings.soft_wrap = self.soft_wrap;
        let _ = save_settings(&self.settings);
    }

    fn sync_heights_to_engine(&mut self, default_h: f32) {
        let target = self.engine.filtered_len();
        if self.row_heights.len() > target {
            self.row_heights.truncate(target);
        } else {
            while self.row_heights.len() < target {
                self.row_heights.push_back(default_h);
            }
        }
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
        self.row_heights.clear();
        if self.find.open {
            self.find.recompute(&self.engine);
        }
    }

    fn start_selected_device(&mut self) {
        self.engine.stop_stream();
        let Some(serial) = self.selected_serial.clone() else {
            self.row_heights.clear();
            return;
        };
        match adb::resolve_adb_path(self.settings.adb_path.as_deref()) {
            Ok(path) => {
                self.row_heights.clear();
                self.row_heights
                    .reserve(self.settings.buffer_capacity.min(16_384));
                self.engine
                    .start_stream(self.runtime_handle(), path, serial);
                self.last_error = None;
            }
            Err(e) => self.last_error = Some(e),
        }
    }

    fn drain_events(&mut self, default_h: f32) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                EngineEvent::RowsAppended(n) => {
                    self.row_heights.reserve(n);
                    for _ in 0..n {
                        self.row_heights.push_back(default_h);
                    }
                    let target = self.engine.filtered_len();
                    if self.row_heights.len() != target {
                        self.sync_heights_to_engine(default_h);
                    }
                    if self.find.open {
                        self.find.append_search(&self.engine, n);
                    }
                }
                EngineEvent::DroppedFront(n) => {
                    for _ in 0..n.min(self.row_heights.len()) {
                        self.row_heights.pop_front();
                    }
                    let target = self.engine.filtered_len();
                    if self.row_heights.len() != target {
                        self.sync_heights_to_engine(default_h);
                    }
                    if self.find.open {
                        self.find.on_dropped_front(n);
                    }
                }
                EngineEvent::Stats(stats) => {
                    self.stats = stats;
                }
                EngineEvent::Error(err) => {
                    self.last_error = Some(err);
                }
                EngineEvent::Cleared => {
                    self.row_heights.clear();
                    self.sync_heights_to_engine(default_h);
                    if self.find.open {
                        self.find.recompute(&self.engine);
                    }
                }
            }
        }
    }

    fn handle_toolbar(&mut self, action: ToolbarAction) {
        match action {
            ToolbarAction::None => {}
            ToolbarAction::RefreshDevices => self.refresh_devices(),
            ToolbarAction::DeviceChanged => self.start_selected_device(),
            ToolbarAction::TogglePause => {
                if self.engine.is_paused() {
                    self.engine.resume();
                    self.row_heights.clear();
                } else {
                    self.engine.pause();
                }
            }
            ToolbarAction::Clear => {
                self.engine.clear_buffer();
                self.row_heights.clear();
            }
            ToolbarAction::ScrollToEndToggled => {
                if self.auto_scroll {
                    self.force_scroll_end = true;
                }
                self.persist_display_prefs();
            }
            ToolbarAction::SoftWrapToggled => {
                self.last_wrap_width = 0.0;
                self.persist_display_prefs();
            }
            ToolbarAction::ExportFiltered => self.export_logs(true),
            ToolbarAction::ExportAll => self.export_logs(false),
        }
    }

    fn export_logs(&mut self, filtered_only: bool) {
        let dialog = rfd::FileDialog::new()
            .set_title(if filtered_only {
                "Export filtered logs"
            } else {
                "Export all logs"
            })
            .add_filter("Log", &["log", "txt"])
            .set_file_name("ohmylogcat.log");

        if let Some(path) = dialog.save_file() {
            let path_str = path.to_string_lossy().to_string();
            match self.engine.export_to_file(&path_str, filtered_only) {
                Ok(()) => {
                    self.status_message = Some(format!("Exported to {}", path_str));
                }
                Err(e) => self.last_error = Some(e),
            }
        }
    }
}

impl eframe::App for OhmylogcatApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Resolve font id outside fonts() — nesting ctx.fonts + ctx.style deadlocks egui.
        let font_id = TextStyle::Monospace.resolve(&ctx.style());
        let default_h = ctx.fonts(|f| f.row_height(&font_id)) + 4.0;
        self.drain_events(default_h);
        self.sync_heights_to_engine(default_h);

        const FILTER_DEBOUNCE: Duration = Duration::from_millis(200);
        if self.filter_dirty {
            let due = self
                .filter_changed_at
                .map(|t| t.elapsed() >= FILTER_DEBOUNCE)
                .unwrap_or(true);
            if due {
                self.apply_filter();
                self.filter_dirty = false;
                self.filter_changed_at = None;
            } else {
                ctx.request_repaint_after(Duration::from_millis(50));
            }
        }

        if self.last_device_refresh.elapsed() > Duration::from_secs(5) {
            self.refresh_devices();
        }

        ui::find_bar::handle_find_shortcuts(ctx, &mut self.find);

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            let prev_show = self.show_settings;
            let action = show_toolbar(
                ui,
                &self.devices,
                &mut self.selected_serial,
                self.engine.is_paused(),
                &mut self.auto_scroll,
                &mut self.soft_wrap,
                &mut self.show_settings,
            );
            if self.show_settings && !prev_show {
                self.settings_panel = SettingsPanelState::from_settings(&self.settings);
            }
            self.handle_toolbar(action);

            ui.separator();
            if show_filter_bar(
                ui,
                &mut self.filter_tag,
                &mut self.filter_message,
                &mut self.filter_level,
            ) {
                self.filter_dirty = true;
                self.filter_changed_at = Some(Instant::now());
            }

            if self.find.open {
                ui.separator();
                if show_find_bar(ui, &mut self.find) {
                    self.find.recompute(&self.engine);
                }
            }

            if let Some(ref err) = self.last_error {
                ui.colored_label(egui::Color32::RED, err);
            }
            if let Some(ref msg) = self.status_message {
                ui.label(msg);
            }
        });

        if let Some(settings) = show_settings_panel(
            ctx,
            &mut self.show_settings,
            &mut self.settings_panel,
            self.auto_scroll,
            self.soft_wrap,
        ) {
            self.settings.adb_path = settings.adb_path;
            self.settings.buffer_capacity = settings.buffer_capacity;
            self.engine.set_capacity(settings.buffer_capacity);
            self.row_heights.clear();
            let _ = save_settings(&self.settings);
        }

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            show_status_bar(ui, self.engine.is_streaming(), &self.stats);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let find_suspends = self.find.is_active_with_matches();
            let stick = self.auto_scroll && !find_suspends;
            let row_count = self.engine.filtered_len();
            let scroll_to_row = self.find.scroll_to_match.take().or_else(|| {
                if self.force_scroll_end {
                    self.force_scroll_end = false;
                    row_count.checked_sub(1)
                } else {
                    None
                }
            });

            let heights = self.row_heights.make_contiguous();
            let engine = self.engine.clone();
            let response = show_log_list(
                ui,
                row_count,
                |start, end| engine.copy_filtered_range(start, end),
                self.soft_wrap,
                heights,
                &mut self.last_wrap_width,
                stick,
                scroll_to_row,
                if self.find.open {
                    &self.find.query
                } else {
                    ""
                },
                if self.find.open {
                    self.find.current_row()
                } else {
                    None
                },
                &mut self.prev_scroll_offset,
            );

            if response.scrolled_away && self.auto_scroll && !find_suspends {
                self.auto_scroll = false;
                self.persist_display_prefs();
            }
        });

        if self.engine.is_streaming() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
}

fn configure_style(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    ctx.set_style(style);
}
