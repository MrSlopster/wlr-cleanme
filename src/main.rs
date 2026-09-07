use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Fullscreen, Window, WindowId};

/// Cycle order: BLACK -> RED -> GREEN -> BLUE -> WHITE -> BLACK ...
/// softbuffer pixels are 0x00RRGGBB (u32).
const COLORS: [u32; 5] = [
    0x000000, // BLACK
    0xFF0000, // RED
    0x00FF00, // GREEN
    0x0000FF, // BLUE
    0xFFFFFF, // WHITE
];

struct App {
    context: Option<Context<OwnedDisplayHandle>>,
    window: Option<Arc<Window>>,
    surface: Option<Surface<OwnedDisplayHandle, Arc<Window>>>,
    color_index: usize,
}

impl App {
    fn new() -> Self {
        Self {
            context: None,
            window: None,
            surface: None,
            color_index: 0,
        }
    }

    fn advance_color(&mut self) {
        self.color_index = (self.color_index + 1) % COLORS.len();
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn draw(&mut self) {
        let (Some(window), Some(surface)) = (self.window.as_ref(), self.surface.as_mut()) else {
            return;
        };
        let size = window.inner_size();
        let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return;
        };
        if surface.resize(width, height).is_err() {
            return;
        }
        let Ok(mut buffer) = surface.buffer_mut() else {
            return;
        };
        buffer.fill(COLORS[self.color_index]);
        let _ = buffer.present();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);
        if self.window.is_some() {
            return;
        }
        let window: Arc<Window> = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("wlr-cleanme")
                        .with_fullscreen(Some(Fullscreen::Borderless(None))),
                )
                .expect("failed to create fullscreen window"),
        );
        window.set_cursor_visible(false);

        let context = Context::new(event_loop.owned_display_handle())
            .expect("failed to create softbuffer context");
        let surface = Surface::new(&context, Arc::clone(&window))
            .expect("failed to create softbuffer surface");

        self.window = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state: ElementState::Pressed,
                        ..
                    },
                is_synthetic: false,
                ..
            } => match logical_key.as_ref() {
                Key::Named(NamedKey::Escape) => {
                    event_loop.exit();
                }
                Key::Named(NamedKey::Space) => {
                    self.advance_color();
                }
                Key::Character("q") | Key::Character("Q") | Key::Character(" ") => {
                    // " " is a fallback for Space on layouts where it arrives as text;
                    // "q"/"Q" quits immediately like Escape.
                    if logical_key == Key::Character(" ".into()) {
                        self.advance_color();
                    } else {
                        event_loop.exit();
                    }
                }
                _ => {}
            },
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left | MouseButton::Right | MouseButton::Middle,
                ..
            } => {
                self.advance_color();
            }
            WindowEvent::Resized(_) => {
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)
}
