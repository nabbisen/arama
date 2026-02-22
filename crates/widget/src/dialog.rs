use iced::{
    Border, Color, Element,
    Length::{self, Fill},
    widget::{container, mouse_area, opaque, space, stack},
};

pub mod media_focus;
pub mod settings;

pub fn overlay<'a, Msg: Clone + 'a>(
    base: Element<'a, Msg>,
    content: Option<Element<'a, Msg>>,
    on_backdrop: Option<Msg>,
) -> Element<'a, Msg> {
    let Some(dialog) = content else { return base };

    // 半透明背景
    let dim = backdrop();

    // 修正ポイント: opaque と mouse_area の順番を入れ替えました
    let background: Element<Msg> = match on_backdrop {
        // mouse_area でクリックを拾い、その外側を opaque で覆って突き抜けを防ぐ
        Some(msg) => opaque(mouse_area(dim).on_press(msg)).into(),
        None => opaque(dim).into(),
    };

    // dialog 自体は opaque で包むことで、その領域のクリックを dialog 側で止める
    // （dialogより外側の領域のクリックだけが、上の background レイヤーに到達する）
    let dialog_centered = container(opaque(dialog)).center(Length::Fill);

    stack![opaque(base), background, dialog_centered].into()
}

fn backdrop<'a, Msg: 'a>() -> Element<'a, Msg> {
    container(space().width(Fill).height(Fill))
        .width(Fill)
        .height(Fill)
        .style(|_| container::Style {
            background: Some(
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.5,
                }
                .into(),
            ),
            ..Default::default()
        })
        .into()
}

pub fn card_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Color::WHITE.into()),
        border: Border {
            color: Color::from_rgb(0.7, 0.7, 0.7),
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: iced::Shadow {
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.3,
            },
            offset: iced::Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        ..Default::default()
    }
}
