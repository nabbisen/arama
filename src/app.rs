pub(super) mod components;
mod image_tensor;
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
