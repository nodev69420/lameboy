use engine::Engine;
mod app;
mod emulator;
mod engine;

pub fn run() {
    app::run::<Engine>();
}
