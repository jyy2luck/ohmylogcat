use crate::engine::Engine;
use crate::ui::format_log_line;
use egui::{Key, Modifiers, Ui};

#[derive(Debug, Default)]
pub struct FindState {
    pub open: bool,
    pub query: String,
    pub matches: Vec<usize>,
    pub current: usize,
    pub focus_request: bool,
    pub scroll_to_match: Option<usize>,
}

impl FindState {
    pub fn open_bar(&mut self) {
        self.open = true;
        self.focus_request = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.matches.clear();
        self.current = 0;
        self.scroll_to_match = None;
    }

    pub fn is_active_with_matches(&self) -> bool {
        self.open && !self.query.trim().is_empty() && !self.matches.is_empty()
    }

    pub fn recompute(&mut self, engine: &Engine) {
        self.matches.clear();
        let q = self.query.trim().to_lowercase();
        if q.is_empty() {
            self.current = 0;
            return;
        }
        let n = engine.filtered_len();
        for i in 0..n {
            if let Some(entry) = engine.filtered_get(i) {
                if format_log_line(&entry).to_lowercase().contains(&q) {
                    self.matches.push(i);
                }
            }
        }
        if self.matches.is_empty() {
            self.current = 0;
        } else if self.current >= self.matches.len() {
            self.current = self.matches.len() - 1;
        }
    }

    /// Scan only the newest `appended` filtered rows (after a live batch).
    pub fn append_search(&mut self, engine: &Engine, appended: usize) {
        let q = self.query.trim().to_lowercase();
        if q.is_empty() || appended == 0 {
            return;
        }
        let n = engine.filtered_len();
        let start = n.saturating_sub(appended);
        for i in start..n {
            if let Some(entry) = engine.filtered_get(i) {
                if format_log_line(&entry).to_lowercase().contains(&q) {
                    self.matches.push(i);
                }
            }
        }
    }

    /// After ring-buffer drops of the oldest filtered rows, shift match indices.
    pub fn on_dropped_front(&mut self, n: usize) {
        if n == 0 {
            return;
        }
        self.matches.retain_mut(|idx| {
            if *idx < n {
                false
            } else {
                *idx -= n;
                true
            }
        });
        if self.matches.is_empty() {
            self.current = 0;
        } else if self.current >= self.matches.len() {
            self.current = self.matches.len() - 1;
        }
    }

    pub fn next(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        self.current = (self.current + 1) % self.matches.len();
        self.scroll_to_match = Some(self.matches[self.current]);
    }

    pub fn prev(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        if self.current == 0 {
            self.current = self.matches.len() - 1;
        } else {
            self.current -= 1;
        }
        self.scroll_to_match = Some(self.matches[self.current]);
    }

    pub fn current_row(&self) -> Option<usize> {
        self.matches.get(self.current).copied()
    }
}

/// Returns true if the query changed (caller should recompute matches).
pub fn show_find_bar(ui: &mut Ui, state: &mut FindState) -> bool {
    let mut query_changed = false;
    ui.horizontal(|ui| {
        ui.label("Find:");
        let response = ui.add(
            egui::TextEdit::singleline(&mut state.query)
                .desired_width(200.0)
                .hint_text("Search logs…"),
        );
        if state.focus_request {
            response.request_focus();
            state.focus_request = false;
        }
        if response.changed() {
            query_changed = true;
        }

        if ui.button("↑").on_hover_text("Previous (Shift+Enter)").clicked() {
            state.prev();
        }
        if ui.button("↓").on_hover_text("Next (Enter)").clicked() {
            state.next();
        }

        let counter = if state.query.trim().is_empty() {
            String::new()
        } else if state.matches.is_empty() {
            "0 matches".into()
        } else {
            format!("{}/{}", state.current + 1, state.matches.len())
        };
        ui.label(counter);

        if ui.button("✕").clicked() {
            state.close();
        }

        if response.has_focus() {
            let enter = ui.input(|i| i.key_pressed(Key::Enter));
            let shift = ui.input(|i| i.modifiers.shift);
            if enter {
                if shift {
                    state.prev();
                } else {
                    state.next();
                }
            }
        }
    });
    query_changed
}

pub fn handle_find_shortcuts(ctx: &egui::Context, state: &mut FindState) {
    let find_pressed = ctx.input(|i| {
        i.modifiers.matches_logically(Modifiers::COMMAND) && i.key_pressed(Key::F)
    });
    if find_pressed {
        state.open_bar();
    }
    if state.open && ctx.input(|i| i.key_pressed(Key::Escape)) {
        state.close();
    }
}
