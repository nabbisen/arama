pub mod message;
pub mod output;
mod update;
mod view;

#[derive(Clone, Debug, Default)]
pub struct Settings {}

// src/dialog/rename.rs ── 完全自己完結

// use super::card_style;
// use iced::{
//     Alignment, Element,
//     widget::{button, column, container, row, text, text_input},
// };

// pub struct State {
//     pub input: String,
// }

// impl State {
//     pub fn new() -> Self {
//         Self {
//             input: String::new(),
//         }
//     }
// }

// #[derive(Clone, Debug)]
// pub enum Message {
//     InputChanged(String),
//     Submit,
//     Cancel,
// }

// pub enum Outcome {
//     Submitted(String),
//     Cancelled,
// }

// // pub fn view(state: &State) -> Element<'_, Message> {
// pub fn view<'a>() -> Element<'a, Message> {
//     text("text")
//         // container(
//         //     column![
//         //         text("名前を変更").size(20),
//         //         text_input("新しい名前...", &state.input).on_input(Message::InputChanged),
//         //         row![
//         //             button("キャンセル").on_press(Message::Cancel),
//         //             button("決定").on_press(Message::Submit),
//         //         ]
//         //         .spacing(8),
//         //     ]
//         //     .spacing(12)
//         //     .align_x(Alignment::End),
//         // )
//         // .padding(24)
//         // .width(360)
//         // .style(card_style)
//         .into()
// }

// // pub fn update(state: &mut State, msg: Message) -> Outcome {
// pub fn update(message: Message) -> Output {
//     match message {
//         // Message::InputChanged(s) => {
//         //     state.input = s;
//         //     Outcome::Cancelled /* 継続 */
//         // }
//         // Message::Submit => Outcome::Submitted(state.input.clone()),
//         // Message::Cancel => Outcome::Cancelled,
//     }
// }
