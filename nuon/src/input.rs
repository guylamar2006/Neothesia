use super::{Node, UpdateCtx, Widget};

#[derive(Clone, Copy, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Clone, Debug)]
pub enum Event {
    CursorMoved { position: dpi::LogicalPosition<f32> },
    MousePress { button: MouseButton },
    MouseRelease { button: MouseButton },
}

impl Event {
    pub fn from_winit(event: &winit::event::WindowEvent, scale_factor: f64) -> Option<Event> {
        fn button_convert(button: winit::event::MouseButton) -> MouseButton {
            match button {
                winit::event::MouseButton::Left => MouseButton::Left,
                winit::event::MouseButton::Right => MouseButton::Right,
                winit::event::MouseButton::Middle => MouseButton::Middle,
                winit::event::MouseButton::Back => MouseButton::Back,
                winit::event::MouseButton::Forward => MouseButton::Forward,
                winit::event::MouseButton::Other(v) => MouseButton::Other(v),
            }
        }

        let event = match event {
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let position = position.to_logical::<f32>(scale_factor);
                Event::CursorMoved { position }
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => match state {
                winit::event::ElementState::Pressed => Event::MousePress {
                    button: button_convert(*button),
                },
                winit::event::ElementState::Released => Event::MouseRelease {
                    button: button_convert(*button),
                },
            },
            _ => return None,
        };

        Some(event)
    }
}

#[derive(Default, Debug)]
pub struct EventQueue {
    event_queue: Vec<Event>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_winit_event(&mut self, event: &winit::event::WindowEvent, scale_factor: f64) {
        if let Some(event) = Event::from_winit(event, scale_factor) {
            self.event_queue.push(event);
        }
    }

    #[profiling::function]
    pub fn dispatch_events<MSG>(
        &mut self,
        messages: &mut Vec<MSG>,
        root: &mut dyn Widget<MSG>,
        layout: &Node,
    ) {
        for event in self.event_queue.drain(..) {
            root.update(
                event,
                layout,
                &mut UpdateCtx {
                    messages,
                    event_captured: false,
                },
            );
        }
    }
}
