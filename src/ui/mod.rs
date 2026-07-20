pub mod find_bar;
mod log_list;
mod settings_panel;

pub use find_bar::{show_find_bar, FindState};
pub use log_list::{format_log_line, show_log_list};
pub use settings_panel::{show_settings_panel, SettingsPanelState};

use crate::adb::Device;
use crate::parser::LogLevel;
use egui::{ComboBox, RichText, Ui};

pub fn show_toolbar(
    ui: &mut Ui,
    devices: &[Device],
    selected_serial: &mut Option<String>,
    is_paused: bool,
    auto_scroll: &mut bool,
    soft_wrap: &mut bool,
    show_settings: &mut bool,
) -> ToolbarAction {
    let mut action = ToolbarAction::None;

    ui.horizontal(|ui| {
        ui.label("Device:");
        let selected_text = selected_serial
            .clone()
            .unwrap_or_else(|| "(none)".into());
        ComboBox::from_id_salt("device_combo")
            .selected_text(&selected_text)
            .width(180.0)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(selected_serial.is_none(), "(none)")
                    .clicked()
                {
                    *selected_serial = None;
                    action = ToolbarAction::DeviceChanged;
                }
                for device in devices {
                    let label = format!("{} ({})", device.serial, device.state);
                    if ui
                        .selectable_label(
                            selected_serial.as_deref() == Some(device.serial.as_str()),
                            label,
                        )
                        .clicked()
                    {
                        *selected_serial = Some(device.serial.clone());
                        action = ToolbarAction::DeviceChanged;
                    }
                }
            });

        if ui.button("↻").on_hover_text("Refresh devices").clicked() {
            action = ToolbarAction::RefreshDevices;
        }

        ui.separator();

        let pause_label = if is_paused { "Resume" } else { "Pause" };
        if ui.button(pause_label).clicked() {
            action = ToolbarAction::TogglePause;
        }

        if ui.button("Clear").clicked() {
            action = ToolbarAction::Clear;
        }

        let scroll_text = if *auto_scroll {
            RichText::new("Scroll to End").strong()
        } else {
            RichText::new("Scroll to End")
        };
        if ui
            .selectable_label(*auto_scroll, scroll_text)
            .on_hover_text("Tail-follow newest logs")
            .clicked()
        {
            *auto_scroll = !*auto_scroll;
            action = ToolbarAction::ScrollToEndToggled;
        }

        let wrap_text = if *soft_wrap {
            RichText::new("Soft-Wrap").strong()
        } else {
            RichText::new("Soft-Wrap")
        };
        if ui
            .selectable_label(*soft_wrap, wrap_text)
            .on_hover_text("Wrap long log lines")
            .clicked()
        {
            *soft_wrap = !*soft_wrap;
            action = ToolbarAction::SoftWrapToggled;
        }

        ui.separator();

        ui.menu_button("Export", |ui| {
            if ui.button("Export filtered…").clicked() {
                action = ToolbarAction::ExportFiltered;
                ui.close_menu();
            }
            if ui.button("Export all…").clicked() {
                action = ToolbarAction::ExportAll;
                ui.close_menu();
            }
        });

        if ui.button("Settings…").clicked() {
            *show_settings = true;
        }
    });

    action
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    None,
    RefreshDevices,
    DeviceChanged,
    TogglePause,
    Clear,
    ScrollToEndToggled,
    SoftWrapToggled,
    ExportFiltered,
    ExportAll,
}

pub fn show_filter_bar(
    ui: &mut Ui,
    tag: &mut String,
    message: &mut String,
    min_level: &mut Option<LogLevel>,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label("Tag:");
        if ui
            .add(egui::TextEdit::singleline(tag).desired_width(120.0))
            .changed()
        {
            changed = true;
        }
        ui.label("Message:");
        if ui
            .add(egui::TextEdit::singleline(message).desired_width(160.0))
            .changed()
        {
            changed = true;
        }
        ui.label("Level:");
        let level_text = min_level
            .map(|l| l.to_display())
            .unwrap_or("Verbose");
        ComboBox::from_id_salt("level_combo")
            .selected_text(level_text)
            .width(90.0)
            .show_ui(ui, |ui| {
                for level in [
                    None,
                    Some(LogLevel::Verbose),
                    Some(LogLevel::Debug),
                    Some(LogLevel::Info),
                    Some(LogLevel::Warn),
                    Some(LogLevel::Error),
                    Some(LogLevel::Fatal),
                ] {
                    let label = level.map(|l| l.to_display()).unwrap_or("Verbose");
                    // None and Verbose both mean "no min filter" / Verbose floor
                    let selected = *min_level == level
                        || (*min_level == Some(LogLevel::Verbose) && level.is_none())
                        || (min_level.is_none() && level == Some(LogLevel::Verbose));
                    if ui.selectable_label(selected, label).clicked() {
                        *min_level = match level {
                            None | Some(LogLevel::Verbose) => None,
                            other => other,
                        };
                        changed = true;
                    }
                }
            });
    });
    changed
}

pub fn show_status_bar(ui: &mut Ui, live: bool, stats: &crate::settings::BufferStats) {
    ui.horizontal(|ui| {
        let (dot, color) = if live {
            ("●", egui::Color32::from_rgb(40, 167, 69))
        } else {
            ("○", egui::Color32::GRAY)
        };
        ui.colored_label(color, dot);
        ui.label(if live { "Live" } else { "Idle" });
        ui.separator();
        ui.label(format!("{}/{}", stats.count, stats.capacity));
        ui.separator();
        ui.label(format!("{:.0} lines/s", stats.lines_per_sec));
        ui.separator();
        ui.label(format!("~{:.1} MB", stats.memory_estimate_mb));
    });
}
