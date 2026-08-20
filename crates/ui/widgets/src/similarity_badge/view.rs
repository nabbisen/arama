use iced::{
    Alignment::Center,
    Background, Border, Element,
    Length::Fill,
    Theme,
    widget::{container, progress_bar, row, text},
};

use super::{MAX_SIMILARITY_RANGE, MIN_SIMILARITY_RANGE, SimilarityBadge};

impl SimilarityBadge {
    fn view<'a, Message>(self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        let s = format!("{:.1} %", self.similarity * 100.0);
        // RFC 043: `body_small` tried here first per the scale, but this
        // is the one site the handoff flagged as possibly needing to stay
        // a literal - the container is a fixed 60px and the worst case
        // ("100.0 %") is tight even at 12px. No `.line_height()`: single
        // line, fixed content, nothing to wrap.
        let label = container(text(s).size(arama_theme::body_small_size())).width(60);

        let bar = progress_bar(MIN_SIMILARITY_RANGE..=MAX_SIMILARITY_RANGE, self.similarity).style(
            move |theme: &Theme| {
                let palette = theme.extended_palette();
                let secondary = palette.secondary;
                iced::widget::progress_bar::Style {
                    background: Background::Color(palette.background.weak.color),
                    bar: Background::Color(secondary.strong.color),
                    border: Border::default(),
                }
            },
        );

        row![label, bar]
            .spacing(5)
            .width(Fill)
            .height(20)
            .padding(2)
            .align_y(Center)
            .into()
    }
}

pub fn similarity_badge<'a, Message>(similarity: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    SimilarityBadge::new(similarity).view::<Message>()
}
