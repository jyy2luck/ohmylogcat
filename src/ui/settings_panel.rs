use crate::settings::{save_settings, BufferPreset, Settings};
use egui::Context;

#[derive(Debug)]
pub struct SettingsPanelState {
    pub adb_path: String,
    pub preset: BufferPreset,
    pub custom_capacity: String,
    pub status: Option<String>,
}

impl SettingsPanelState {
    pub fn from_settings(settings: &Settings) -> Self {
        let preset = BufferPreset::from_capacity(settings.buffer_capacity);
        Self {
            adb_path: settings.adb_path.clone().unwrap_or_default(),
            preset,
            custom_capacity: settings.buffer_capacity.to_string(),
            status: None,
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

/// Returns `Some(partial settings)` when the user saves (adb_path + buffer_capacity).
/// Caller merges display prefs (auto_scroll / soft_wrap) before persisting.
pub fn show_settings_panel(
    ctx: &Context,
    open: &mut bool,
    state: &mut SettingsPanelState,
    auto_scroll_to_end: bool,
    soft_wrap: bool,
) -> Option<Settings> {
    if !*open {
        return None;
    }

    let mut saved = None;
    let mut close = false;
    egui::Window::new("Settings")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("ADB Path:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.adb_path)
                        .desired_width(280.0)
                        .hint_text("Leave empty to use PATH / defaults"),
                );
            });

            ui.add_space(8.0);
            ui.label("Buffer preset:");
            ui.horizontal_wrapped(|ui| {
                for preset in BufferPreset::ALL {
                    if ui
                        .selectable_label(state.preset == preset, preset.label())
                        .clicked()
                    {
                        state.preset = preset;
                        if let Some(cap) = preset.capacity() {
                            state.custom_capacity = cap.to_string();
                        }
                    }
                }
            });

            if state.preset == BufferPreset::Custom {
                ui.horizontal(|ui| {
                    ui.label("Custom capacity:");
                    ui.add(
                        egui::TextEdit::singleline(&mut state.custom_capacity).desired_width(100.0),
                    );
                });
            }

            if let Some(ref status) = state.status {
                ui.label(status.clone());
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    let settings = Settings {
                        adb_path: if state.adb_path.trim().is_empty() {
                            None
                        } else {
                            Some(state.adb_path.trim().to_string())
                        },
                        buffer_capacity: state.capacity(),
                        auto_scroll_to_end,
                        soft_wrap,
                    };
                    match save_settings(&settings) {
                        Ok(()) => {
                            state.status = Some("Settings saved".into());
                            saved = Some(settings);
                            close = true;
                        }
                        Err(e) => state.status = Some(e),
                    }
                }
                if ui.button("Cancel").clicked() {
                    close = true;
                }
            });
        });

    if close {
        *open = false;
    }
    saved
}
