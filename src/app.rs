use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event_loop::{ControlFlow, EventLoop},
};

pub type Window = winit::window::Window;
pub type WindowEvent = winit::event::WindowEvent;
pub type DeviceEvent = winit::event::DeviceEvent;

pub fn run<T: Application>() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app: App<T> = App { state: None };

    event_loop.run_app(&mut app).unwrap();
}

pub enum AppEvent {
    Window(WindowEvent),
    Device(DeviceEvent),
}

#[derive(Debug, Clone, Copy)]
pub enum AppSignal {
    Continue,
    Reload,
    Quit,
}

pub trait Application {
    fn new_app(window: Arc<Window>) -> Self;
    fn handle_event(&mut self, event: &AppEvent) -> AppSignal;
    fn update(&mut self) -> AppSignal;
}

pub struct AppState<T: Application> {
    internal: T,
    window: Arc<Window>,
}

impl<T: Application> AppState<T> {
    pub fn handle_event(
        &mut self,
        event: AppEvent,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        let signal = self.internal.handle_event(&event);
        self.handle_signal(signal, event_loop);

        match event {
            AppEvent::Window(WindowEvent::RedrawRequested) => {
                let signal = self.internal.update();
                self.handle_signal(signal, event_loop);
            }
            _ => {}
        }
    }

    fn handle_signal(
        &mut self,
        signal: AppSignal,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        use AppSignal::*;
        match signal {
            Continue => {}
            Reload => {
                self.internal = T::new_app(self.window.clone());
            }
            Quit => {
                event_loop.exit();
            }
        }
    }
}

pub struct App<T: Application> {
    state: Option<AppState<T>>,
}

impl<T: Application> ApplicationHandler for App<T> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window =
            event_loop.create_window(Window::default_attributes().with_title(
                "TODO: figure out of a way of setting the window name.",
            ));

        match window {
            Ok(window) => {
                let window = Arc::new(window);
                let internal = T::new_app(window.clone());

                self.state = Some(AppState { internal, window });
            }
            Err(err) => {
                eprintln!("critical app error, failed to create window, err: {err:#?}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(state) = &mut self.state {
            state.handle_event(AppEvent::Window(event), event_loop);
        } else {
            eprintln!(
                "critical app error during App::window_event(), None app state"
            );
            event_loop.exit();
        }
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(state) = &mut self.state {
            state.handle_event(AppEvent::Device(event), event_loop);
        } else {
            eprintln!(
                "critical app error during App::device_event(), None app state"
            );
            event_loop.exit();
        }
    }

    fn about_to_wait(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        } else {
            eprintln!("critical app error during App::about_to_wait(), None app state");
            event_loop.exit();
        }
    }
}
