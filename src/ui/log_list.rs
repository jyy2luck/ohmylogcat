use crate::parser::{LogEntry, LogLevel};
use egui::{Color32, Sense, TextStyle, Ui, Vec2};

pub fn format_log_line(entry: &LogEntry) -> String {
    format!(
        "{} {} {} {} {}: {}",
        entry.timestamp,
        entry.pid,
        entry.tid,
        entry.level.to_display().chars().next().unwrap_or('?'),
        entry.tag,
        entry.message
    )
}

pub fn level_color(level: LogLevel) -> Color32 {
    match level {
        LogLevel::Verbose => Color32::from_rgb(150, 150, 150),
        LogLevel::Debug => Color32::from_rgb(100, 140, 200),
        LogLevel::Info => Color32::from_rgb(40, 40, 40),
        LogLevel::Warn => Color32::from_rgb(180, 120, 0),
        LogLevel::Error | LogLevel::Fatal => Color32::from_rgb(200, 40, 40),
    }
}

pub struct LogListResponse {
    pub at_bottom: bool,
    pub scrolled_away: bool,
}

fn estimate_wrapped_height(text_len: usize, wrap_width: f32, line_h: f32) -> f32 {
    let char_w = (line_h * 0.55).max(4.0);
    let cols = (wrap_width / char_w).floor().max(1.0);
    let lines = ((text_len as f32) / cols).ceil().max(1.0);
    lines * line_h + 2.0
}

/// Virtualized log list.
///
/// `fetch_range(start, end)` must return entries for filtered indices `[start, end)`.
pub fn show_log_list(
    ui: &mut Ui,
    row_count: usize,
    mut fetch_range: impl FnMut(usize, usize) -> Vec<LogEntry>,
    soft_wrap: bool,
    row_heights: &mut [f32],
    last_wrap_width: &mut f32,
    stick_to_bottom: bool,
    scroll_to_row: Option<usize>,
    find_query: &str,
    current_match_row: Option<usize>,
    prev_scroll_offset: &mut f32,
) -> LogListResponse {
    let line_h = ui.text_style_height(&TextStyle::Monospace) + 4.0;
    // Caller keeps length in sync with row_count (App::sync_heights_to_engine).
    let row_heights = if row_heights.len() > row_count {
        &mut row_heights[..row_count]
    } else {
        row_heights
    };
    if row_heights.len() < row_count {
        // Length mismatch — paint with what we have; avoid panics.
    }

    if soft_wrap {
        return show_wrapped_virtual(
            ui,
            row_count.min(row_heights.len()),
            &mut fetch_range,
            row_heights,
            last_wrap_width,
            line_h,
            stick_to_bottom,
            scroll_to_row,
            find_query,
            current_match_row,
            prev_scroll_offset,
        );
    }

    let mut scroll = egui::ScrollArea::vertical()
        .id_salt("log_list")
        .auto_shrink([false, false])
        .stick_to_bottom(stick_to_bottom && scroll_to_row.is_none())
        .hscroll(true);

    if let Some(row) = scroll_to_row {
        let y = (row as f32 * line_h - line_h * 2.0).max(0.0);
        scroll = scroll.vertical_scroll_offset(y);
    }

    let output = scroll.show_rows(ui, line_h, row_count.max(1), |ui, row_range| {
        if row_count == 0 {
            ui.weak("No log entries");
            return;
        }
        let start = row_range.start;
        let end = row_range.end.min(row_count);
        let entries = fetch_range(start, end);
        for (offset, entry) in entries.into_iter().enumerate() {
            let row = start + offset;
            paint_row(
                ui,
                &entry,
                find_query,
                current_match_row == Some(row),
                false,
            );
        }
    });

    finish_scroll(output, prev_scroll_offset)
}

fn show_wrapped_virtual(
    ui: &mut Ui,
    row_count: usize,
    fetch_range: &mut impl FnMut(usize, usize) -> Vec<LogEntry>,
    row_heights: &mut [f32],
    last_wrap_width: &mut f32,
    line_h: f32,
    stick_to_bottom: bool,
    scroll_to_row: Option<usize>,
    find_query: &str,
    current_match_row: Option<usize>,
    prev_scroll_offset: &mut f32,
) -> LogListResponse {
    let wrap_width = ui.available_width().max(1.0);
    let row_count = row_count.min(row_heights.len());

    if (*last_wrap_width - wrap_width).abs() > 1.0 {
        let default_est = estimate_wrapped_height(160, wrap_width, line_h);
        row_heights.fill(default_est);
        *last_wrap_width = wrap_width;
    }

    let mut scroll = egui::ScrollArea::vertical()
        .id_salt("log_list_wrap")
        .auto_shrink([false, false])
        .stick_to_bottom(stick_to_bottom && scroll_to_row.is_none());

    if let Some(row) = scroll_to_row {
        let y: f32 = row_heights.iter().take(row).sum::<f32>() - line_h * 2.0;
        scroll = scroll.vertical_scroll_offset(y.max(0.0));
    }

    let output = scroll.show_viewport(ui, |ui, viewport| {
        if row_count == 0 {
            ui.weak("No log entries");
            return;
        }

        let content_width = ui.available_width().max(1.0);
        let overscan = line_h * 8.0;

        let mut start = 0usize;
        let mut y_before = 0.0f32;
        while start < row_count {
            let h = row_heights[start];
            if y_before + h >= viewport.min.y - overscan {
                break;
            }
            y_before += h;
            start += 1;
        }

        if y_before > 0.0 {
            ui.allocate_exact_size(Vec2::new(content_width, y_before), Sense::hover());
        }

        // Estimate how many rows fit in the viewport + overscan.
        let mut end = start;
        let mut y = y_before;
        while end < row_count && y < viewport.max.y + overscan {
            y += row_heights[end];
            end += 1;
        }
        end = end.min(row_count);

        let entries = fetch_range(start, end);
        let mut y = y_before;
        for (offset, entry) in entries.into_iter().enumerate() {
            let row = start + offset;
            let response = paint_row(
                ui,
                &entry,
                find_query,
                current_match_row == Some(row),
                true,
            );
            let actual = response.rect.height().max(line_h * 0.5);
            if row < row_heights.len() {
                row_heights[row] = actual;
            }
            y += actual;
            let _ = y;
        }

        let painted_end = start + (end - start).min(row_count.saturating_sub(start));
        let rest: f32 = row_heights.get(painted_end..).map(|s| s.iter().sum()).unwrap_or(0.0);
        if rest > 0.0 {
            ui.allocate_exact_size(Vec2::new(content_width, rest), Sense::hover());
        }
    });

    finish_scroll(output, prev_scroll_offset)
}

fn paint_row(
    ui: &mut Ui,
    entry: &LogEntry,
    find_query: &str,
    is_current: bool,
    wrap: bool,
) -> egui::Response {
    let text = format_log_line(entry);
    let color = level_color(entry.level);
    let mut job = egui::text::LayoutJob::default();
    append_highlighted(&mut job, &text, find_query, color, is_current, ui.style());
    if wrap {
        ui.add(egui::Label::new(job).wrap())
    } else {
        ui.add(egui::Label::new(job).extend())
    }
}

fn finish_scroll(
    output: egui::scroll_area::ScrollAreaOutput<()>,
    prev_scroll_offset: &mut f32,
) -> LogListResponse {
    let max_offset = (output.content_size.y - output.inner_rect.height()).max(0.0);
    let offset = output.state.offset.y;
    let at_bottom = offset >= max_offset - 2.0;
    let scrolled_away = offset < *prev_scroll_offset - 1.0 && !at_bottom;
    *prev_scroll_offset = offset;
    LogListResponse {
        at_bottom,
        scrolled_away,
    }
}

fn append_highlighted(
    job: &mut egui::text::LayoutJob,
    text: &str,
    query: &str,
    base_color: Color32,
    emphasize_row: bool,
    style: &egui::Style,
) {
    let font_id = TextStyle::Monospace.resolve(style);
    let row_bg = if emphasize_row {
        Color32::from_rgb(255, 220, 120)
    } else {
        Color32::TRANSPARENT
    };

    let q = query.trim();
    if q.is_empty() {
        job.append(
            text,
            0.0,
            egui::text::TextFormat {
                font_id,
                color: base_color,
                background: row_bg,
                ..Default::default()
            },
        );
        return;
    }

    let lower_text = text.to_lowercase();
    let lower_q = q.to_lowercase();
    let mut start = 0usize;
    while let Some(pos) = lower_text[start..].find(&lower_q) {
        let abs = start + pos;
        if abs > start {
            job.append(
                &text[start..abs],
                0.0,
                egui::text::TextFormat {
                    font_id: font_id.clone(),
                    color: base_color,
                    background: row_bg,
                    ..Default::default()
                },
            );
        }
        let mut match_end = abs + lower_q.len();
        while match_end > abs && !text.is_char_boundary(match_end) {
            match_end -= 1;
        }
        match_end = match_end.min(text.len());
        job.append(
            &text[abs..match_end],
            0.0,
            egui::text::TextFormat {
                font_id: font_id.clone(),
                color: Color32::BLACK,
                background: Color32::from_rgb(255, 235, 59),
                ..Default::default()
            },
        );
        start = match_end;
        if start >= text.len() {
            break;
        }
    }
    if start < text.len() {
        job.append(
            &text[start..],
            0.0,
            egui::text::TextFormat {
                font_id,
                color: base_color,
                background: row_bg,
                ..Default::default()
            },
        );
    }
}
