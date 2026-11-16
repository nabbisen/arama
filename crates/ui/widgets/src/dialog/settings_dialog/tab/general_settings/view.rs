use arama_env::target_media_type::TargetMediaType;
use arama_env::{TextScale, ThemePreset};
use arama_i18n::{Locale, t};
use iced::{
    Element,
    widget::{button, checkbox, column, container, row, slider, text},
};
use lucide_icons::iced::{icon_arrow_down, icon_arrow_up};

use super::{GeneralSettings, message::Message};

impl GeneralSettings {
    pub fn view(&self) -> Element<'_, Message> {
        let target_media_types = row![
            checkbox(self.target_media_type.include_image)
                .label(t("settings.general.include_image"))
                .on_toggle(|x| {
                    Message::TargetMediaTypeChanged(TargetMediaType {
                        include_image: x,
                        include_video: self.target_media_type.include_video,
                    })
                }),
            checkbox(self.target_media_type.include_video)
                .label(t("settings.general.include_video"))
                .on_toggle(|x| {
                    Message::TargetMediaTypeChanged(TargetMediaType {
                        include_image: self.target_media_type.include_image,
                        include_video: x,
                    })
                })
        ]
        .spacing(10)
        .wrap();

        let sub_dir_depth_limit = row![
            text(t("settings.general.sub_dir_depth")),
            button(icon_arrow_down().size(12))
                .padding(2)
                .on_press_maybe(if 0 < self.sub_dir_depth_limit {
                    Some(Message::SubDirDepthLimitChanged(
                        self.sub_dir_depth_limit - 1,
                    ))
                } else {
                    None
                }),
            text(self.sub_dir_depth_limit),
            button(icon_arrow_up().size(12)).padding(2).on_press_maybe(
                if self.sub_dir_depth_limit < 2 {
                    Some(Message::SubDirDepthLimitChanged(
                        self.sub_dir_depth_limit + 1,
                    ))
                } else {
                    None
                }
            ),
        ]
        .spacing(5);

        let threshold_slider = row![
            text(t("settings.general.similarity")),
            text("0.50").style(text::secondary),
            slider(0.50_f32..=1.00_f32, self.similarity_threshold, |v| {
                Message::SimilarityThresholdChanged((v * 100.0).round() / 100.0)
            })
            .step(0.01_f32),
            text("1.00").style(text::secondary),
            text(format!("{:.2}", self.similarity_threshold)),
        ]
        .spacing(8);

        // T063: `.wrap()` (iced 0.14's native wrapping row) rather than a
        // plain `row!`, found necessary once Larger text made these
        // overflow the 600px-wide settings dialog box horizontally at
        // 760x520 - a real plant (`ja.rs`) showed
        // "ハイコントラスト（暗）" pushed past x=600 with the row's own
        // alignment otherwise unchanged.
        let locale_buttons = Locale::all()
            .iter()
            .fold(
                row![text(t("settings.general.language"))].spacing(8),
                |row, locale| {
                    let btn = button(locale.display_name())
                        .style(if &self.locale == locale {
                            arama_theme::primary
                        } else {
                            arama_theme::ghost
                        })
                        .on_press(Message::LocaleChanged(*locale));
                    row.push(btn)
                },
            )
            .wrap();

        let theme_buttons = ThemePreset::all()
            .iter()
            .fold(
                row![text(t("settings.general.theme"))].spacing(8),
                |row, preset| {
                    let label = match preset {
                        ThemePreset::Light => t("settings.general.theme.light"),
                        ThemePreset::Dark => t("settings.general.theme.dark"),
                        ThemePreset::HighContrastLight => t("settings.general.theme.hc_light"),
                        ThemePreset::HighContrastDark => t("settings.general.theme.hc_dark"),
                    };
                    let btn = button(text(label))
                        .style(if &self.theme == preset {
                            arama_theme::primary
                        } else {
                            arama_theme::ghost
                        })
                        .on_press(Message::ThemeChanged(*preset));
                    row.push(btn)
                },
            )
            .wrap();

        // RFC 043 §2: this note explains the high-contrast themes, so
        // rendering it at 12px with reduced contrast worked against the
        // accessibility feature it describes. `body_small` fits its role
        // (secondary, skimmable help text) without compounding that with
        // `text::secondary`'s own contrast reduction on top.
        let theme_note = text(t("settings.general.theme.hc_note"))
            .size(arama_theme::body_small_size())
            .line_height(arama_theme::body_small_line_height());

        let text_scale_buttons = TextScale::all()
            .iter()
            .fold(
                row![text(t("settings.general.text_size"))].spacing(8),
                |row, scale| {
                    let label = match scale {
                        TextScale::Default => t("settings.general.text_size.default"),
                        TextScale::Large => t("settings.general.text_size.large"),
                        TextScale::Larger => t("settings.general.text_size.larger"),
                    };
                    let btn = button(text(label))
                        .style(if &self.text_scale == scale {
                            arama_theme::primary
                        } else {
                            arama_theme::ghost
                        })
                        .on_press(Message::TextScaleChanged(*scale));
                    row.push(btn)
                },
            )
            .wrap();

        // RFC 052 §2.2: text size is read once at startup
        // (`App::settings()`'s `default_text_size`), unlike theme and
        // locale, which both take effect immediately - so a changed
        // choice needs its own, visible notice. `body_small` for the
        // same reason as `theme_note` just above: this note states a
        // real behavior the reader needs to act on, not filler.
        let text_scale_note = text(t("settings.general.text_size.restart_notice"))
            .size(arama_theme::body_small_size())
            .line_height(arama_theme::body_small_line_height());

        container(
            column![
                target_media_types,
                sub_dir_depth_limit,
                threshold_slider,
                locale_buttons,
                theme_buttons,
                theme_note,
                text_scale_buttons,
                text_scale_note
            ]
            .spacing(10),
        )
        .into()
    }
}
