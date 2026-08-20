use arama_i18n::{Locale, current_locale, t};
use chrono::{Local, TimeZone};
use iced::{
    Element,
    Length::{Fill, FillPortion, Fixed},
    widget::{button, column, container, row, scrollable, text, text_input},
};
use lucide_icons::iced::{icon_circle_stop, icon_refresh_cw, icon_scissors, icon_trash_2};

use super::{
    CachePage, DirRow,
    message::{Event, Internal, Message},
    parse_mib_target,
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

        // ── Footprint and explicit prune controls ─────────────────────
        let footprint = self
            .footprint
            .map(|f| human_size(f.total_bytes))
            .unwrap_or_else(|| t("cache.footprint.unavailable"));
        let prune_target_valid = parse_mib_target(&self.prune_target_input).is_some();
        let prune_row = row![
            text(format!("{}: {footprint}", t("cache.footprint"))),
            text_input(&t("cache.prune.placeholder"), &self.prune_target_input)
                .on_input(|s| Message::Internal(Internal::PruneTargetInput(s)))
                .width(Fixed(120.0)),
            text(t("cache.prune.unit_mib")).style(text::secondary),
            button(row![icon_scissors().size(14), text(t("cache.prune.button"))].spacing(4))
                .on_press_maybe(if run_active || self.prune_busy || !prune_target_valid {
                    None
                } else {
                    Some(Message::Internal(Internal::PrunePressed))
                }),
        ]
        .spacing(10);

        let prune_result: Element<'_, Message> = self
            .last_prune_report
            .as_ref()
            .map(|report| {
                text(prune_report_text(report))
                    .style(text::secondary)
                    .into()
            })
            .unwrap_or_else(|| text("").into());

        let load_error: Element<'_, Message> = self
            .load_error
            .as_ref()
            .map(|message| {
                let prefix = if self.rows.is_empty() {
                    t("cache.load_error")
                } else {
                    t("cache.load_error.stale")
                };
                // Arbitrary-length underlying error, same reasoning as
                // the other dynamic error sites in this RFC's pass.
                text(format!("{prefix}: {message}"))
                    .size(arama_theme::body_size())
                    .line_height(arama_theme::body_line_height())
                    .into()
            })
            .unwrap_or_else(|| text("").into());

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

        column![
            add_form,
            prune_row,
            prune_result,
            load_error,
            filter_row,
            table,
            summary
        ]
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
            text(format_relative_timestamp(r.latest_cached_at)).into()
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

        // RFC 043: a path is the one string where a misread character
        // changes the meaning - `body_small` over the old 13px literal,
        // with a line-height since a long path can still wrap in this
        // fixed-portion column.
        row![
            container(
                text(&r.dir_path)
                    .size(arama_theme::body_small_size())
                    .line_height(arama_theme::body_small_line_height())
            )
            .width(FillPortion(5)),
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

fn prune_report_text(report: &arama_cache::CachePruneReport) -> String {
    if report.target_reached {
        format!(
            "{}: {} {} · {} {}",
            t("cache.prune.done"),
            report.removed_entries,
            t("cache.prune.entries"),
            t("cache.footprint"),
            human_size(report.after.total_bytes),
        )
    } else {
        format!(
            "{}: {} {} · {} {} · {} {}",
            t("cache.prune.partial"),
            report.removed_entries,
            t("cache.prune.entries"),
            t("cache.footprint"),
            human_size(report.after.total_bytes),
            t("cache.prune.unreclaimable"),
            human_size(report.unreclaimable_bytes),
        )
    }
}

/// Relative local time, such as `2 days ago`.
fn format_relative_timestamp(unix_secs: i64) -> String {
    format_relative_timestamp_at(unix_secs, Local::now().timestamp())
}

fn format_relative_timestamp_at(unix_secs: i64, now_unix_secs: i64) -> String {
    if unix_secs <= 0 || Local.timestamp_opt(unix_secs, 0).single().is_none() {
        return "\u{2014}".to_owned();
    }

    let elapsed_secs = now_unix_secs.saturating_sub(unix_secs);
    if elapsed_secs < 60 {
        return t("cache.time.just_now");
    }

    let (value, singular_key, plural_key) = if elapsed_secs < 60 * 60 {
        (elapsed_secs / 60, "cache.time.minute", "cache.time.minutes")
    } else if elapsed_secs < 60 * 60 * 24 {
        (
            elapsed_secs / (60 * 60),
            "cache.time.hour",
            "cache.time.hours",
        )
    } else if elapsed_secs < 60 * 60 * 24 * 30 {
        (
            elapsed_secs / (60 * 60 * 24),
            "cache.time.day",
            "cache.time.days",
        )
    } else if elapsed_secs < 60 * 60 * 24 * 365 {
        (
            elapsed_secs / (60 * 60 * 24 * 30),
            "cache.time.month",
            "cache.time.months",
        )
    } else {
        (
            elapsed_secs / (60 * 60 * 24 * 365),
            "cache.time.year",
            "cache.time.years",
        )
    };

    let unit = if value == 1 {
        t(singular_key)
    } else {
        t(plural_key)
    };
    let ago = t("cache.time.ago");

    match current_locale() {
        Locale::Ja => format!("{value}{unit}{ago}"),
        Locale::En => format!("{value} {unit} {ago}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arama_i18n::{Locale, set_locale};

    #[test]
    fn relative_timestamp_formats_ranges_and_locales() {
        set_locale(Locale::En);
        let now = 1_700_000_000;

        assert_eq!(format_relative_timestamp_at(0, now), "\u{2014}");
        assert_eq!(format_relative_timestamp_at(now - 30, now), "just now");
        assert_eq!(format_relative_timestamp_at(now - 60, now), "1 minute ago");
        assert_eq!(
            format_relative_timestamp_at(now - 2 * 60, now),
            "2 minutes ago"
        );
        assert_eq!(
            format_relative_timestamp_at(now - 60 * 60, now),
            "1 hour ago"
        );
        assert_eq!(
            format_relative_timestamp_at(now - 2 * 60 * 60, now),
            "2 hours ago"
        );
        assert_eq!(
            format_relative_timestamp_at(now - 2 * 60 * 60 * 24, now),
            "2 days ago"
        );

        set_locale(Locale::Ja);
        assert_eq!(
            format_relative_timestamp_at(now - 2 * 60 * 60 * 24, now),
            "2\u{65e5}\u{524d}"
        );
        set_locale(Locale::En);
    }
}
