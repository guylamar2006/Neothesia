pub mod widget;
pub use widget::*;

pub use euclid;

pub type Point = euclid::default::Point2D<f32>;
pub type Size = euclid::default::Size2D<f32>;
pub type Box2D = euclid::default::Box2D<f32>;
pub type Rect = euclid::default::Rect<f32>;

pub mod input;
mod renderer;

pub use input::{Event, MouseButton};
pub use renderer::Renderer;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<[f32; 4]> for Color {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Self { r, g, b, a }
    }
}

impl Color {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn new_u8(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a,
        }
    }
}

pub struct RenderCtx {}

pub struct UpdateCtx<'a, MSG> {
    pub messages: &'a mut Vec<MSG>,
    pub event_captured: bool,
}

impl<MSG> UpdateCtx<'_, MSG> {
    pub fn capture_event(&mut self) {
        self.event_captured = true;
    }

    pub fn is_event_captured(&self) -> bool {
        self.event_captured
    }
}

#[derive(Default, Clone)]
pub struct LayoutCtx {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub trait Widget<MSG> {
    fn layout(&self, ctx: &LayoutCtx) -> Node;
    fn render(&self, renderer: &mut dyn Renderer, layout: &Node, ctx: &RenderCtx);
    fn update(&mut self, event: input::Event, layout: &Node, ctx: &mut UpdateCtx<MSG>);
}

#[derive(Default, Clone)]
pub struct Node {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub children: Vec<Node>,
}

impl Node {
    pub fn as_rect(&self) -> crate::Rect {
        crate::Rect::new((self.x, self.y).into(), (self.w, self.h).into())
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.as_rect().contains((x, y).into())
    }
}

pub struct Element<'a, MSG>(Box<dyn Widget<MSG> + 'a>);

impl<'a, MSG> Element<'a, MSG> {
    pub fn new(widget: impl Widget<MSG> + 'a) -> Self {
        Self(Box::new(widget))
    }

    pub fn as_widget(&self) -> &dyn Widget<MSG> {
        self.0.as_ref()
    }

    pub fn as_widget_mut(&mut self) -> &mut dyn Widget<MSG> {
        self.0.as_mut()
    }
}
