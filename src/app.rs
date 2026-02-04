mod gallery;
mod util;
mod window;

pub fn start() -> iced::Result {
    iced::application(
        window::Window::new,
        window::Window::update,
        window::Window::view,
    )
    .run()
}
