use arama_i18n::t;
use chrono::{Local, TimeZone};
use iced::{
    Element,
    Length::{Fill, FillPortion},
    widget::{button, column, container, row, scrollable, text, text_input},
};
use lucide_icons::iced::{icon_circle_stop, icon_refresh_cw, icon_trash_2};

use super::{
    CachePage, DirRow,
    message::{Event, Internal, Message},
};

impl CachePage {
    pub fn view(&self) -> Element<'_, Message> {
        let run_active = self.active_run.is_some();

        // ── Add-directory form ────────────────────────────────────────
        let add_form = row![
            text_input(&t("cache.form.placeholder"), &self.dir_input)
                .on_input(|s| Message::Internal(Internal::DirInput(s)))
                .on_submit(Message::Internal(Internal::CachePressed)),
            button(text(t("cache.form.button"))).on_press_maybe(if run_active {
                None
            } else {
                Some(Message::Internal(Internal::CachePressed))
            }),
        ]
        .spacing(10);

        // ── Filter row ────────────────────────────────────────────────
        let filter_row = row![
            text_input(&t("cache.filter.placeholder"), &self.filter)
                .on_input(|s| Message::Internal(Internal::FilterInput(s))),
            button(icon_refresh_cw()).on_press_maybe(if self.busy {
                None
            } else {
                Some(Message::Internal(Internal::RefreshPressed))
            }),
        ]
        .spacing(10);

        // ── Table ─────────────────────────────────────────────────────
        let filter = self.filter.to_lowercase();
        let visible: Vec<&DirRow> = self
            .rows
            .iter()
            .filter(|r| filter.is_empty() || r.dir_path.to_lowercase().contains(&filter))
            .collect();

        let table: Element<'_, Message> = if self.rows.is_empty() {
            container(text(t("cache.empty")).style(text::secondary))
                .padding(20)
                .into()
        } else if visible.is_empty() {
            container(text(t("cache.no_match")).style(text::secondary))
                .padding(20)
                .into()
        } else {
            let header = table_header();
            let body = visible.iter().fold(column![].spacing(2), |acc, r| {
                acc.push(self.table_row(r, run_active))
            });
            column![header, scrollable(body).height(Fill)]
                .spacing(4)
                .into()
        };

        // ── Summary (unfiltered totals) ───────────────────────────────
        let total_files: usize = self.rows.iter().map(|r| r.file_count).sum();
        let total_size: u64 = self.rows.iter().map(|r| r.total_size).sum();
        let summary = text(format!(
            "{} {} · {} {} · {} {}",
            self.rows.len(),
            t("cache.summary.directories"),
            total_files,
            t("cache.summary.files"),
            human_size(total_size),
            t("cache.summary.total"),
        ))
        .style(text::secondary);

        column![add_form, filter_row, table, summary]
            .spacing(15)
            .padding(20)
            .into()
    }

    fn table_row<'a>(&self, r: &'a DirRow, run_active: bool) -> Element<'a, Message> {
        let is_running = self
            .active_run
            .as_deref()
            .map(|p| p.to_string_lossy() == r.dir_path)
            .unwrap_or(false);

        let cached_at_col: Element<'_, Message> = if is_running {
            // ⏳ caching… + stop button
            row![
                text(t("cache.row.caching")),
                button(icon_circle_stop().size(14))
                    .padding(4)
                    .style(arama_theme::danger)
                    .on_press(Message::Event(Event::StopRequest)),
            ]
            .spacing(6)
            .into()
        } else {
            text(format_timestamp(r.latest_cached_at)).into()
        };

        let clear = button(icon_trash_2().size(14))
            .padding(4)
            .on_press_maybe(if run_active {
                None
            } else {
                Some(Message::Event(Event::ClearRequest(
                    r.dir_path.clone().into(),
                )))
            });

        row![
            container(text(&r.dir_path).size(13)).width(FillPortion(5)),
            container(text(r.file_count)).width(FillPortion(1)),
            container(text(human_size(r.total_size))).width(FillPortion(1)),
            container(cached_at_col).width(FillPortion(2)),
            clear,
        ]
        .spacing(10)
        .padding([4, 0])
        .into()
    }
}

fn table_header<'a>() -> Element<'a, Message> {
    row![
        container(text(t("cache.column.directory")).style(text::secondary)).width(FillPortion(5)),
        container(text(t("cache.column.files")).style(text::secondary)).width(FillPortion(1)),
        container(text(t("cache.column.size")).style(text::secondary)).width(FillPortion(1)),
        container(text(t("cache.column.cached_at")).style(text::secondary)).width(FillPortion(2)),
        container(text("")).width(30),
    ]
    .spacing(10)
    .into()
}

/// `41.2 MB`-style humanised size (binary-1024 units).
fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while 1024.0 <= value && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

/// Absolute local time, `YYYY-MM-DD HH:MM`.
fn format_timestamp(unix_secs: i64) -> String {
    match Local.timestamp_opt(unix_secs, 0).single() {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => "\u{2014}".to_owned(),
    }
}
