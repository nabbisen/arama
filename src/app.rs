pub(super) mod components;
mod utils;
mod views;
mod window;

pub fn start() -> iced::Result {
    iced::application(
        window::Window::new,
        window::Window::update,
        window::Window::view,
    )
    .run()
}
