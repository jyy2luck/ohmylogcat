use crate::engine::Engine;
use crate::ui::format_log_line;
use crate::ui::TextInput;

#[derive(Debug, Default)]
pub struct FindState {
    pub open: bool,
    pub input: TextInput,
    pub matches: Vec<usize>,
    pub current: usize,
    pub scroll_to_match: Option<usize>,
}

impl FindState {
    pub fn open_bar(&mut self) {
        self.open = true;
        self.input.set_cursor_end();
    }

    pub fn close(&mut self) {
        self.open = false;
        self.input = TextInput::default();
        self.matches.clear();
        self.current = 0;
        self.scroll_to_match = None;
    }

    pub fn is_active_with_matches(&self) -> bool {
        self.open && !self.input.text.trim().is_empty() && !self.matches.is_empty()
    }

    pub fn recompute(&mut self, engine: &Engine) {
        self.matches.clear();
        let q = self.input.text.trim().to_lowercase();
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

    pub fn append_search(&mut self, engine: &Engine, appended: usize) {
        let q = self.input.text.trim().to_lowercase();
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

    pub fn counter_text(&self) -> String {
        if self.input.text.trim().is_empty() {
            String::new()
        } else if self.matches.is_empty() {
            "0 matches".into()
        } else {
            format!("{}/{}", self.current + 1, self.matches.len())
        }
    }
}
