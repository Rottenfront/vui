mod align;
pub use align::*;
mod binding;
pub use binding::*;
mod colors;
pub use colors::*;
mod context;
pub use context::*;
mod event;
pub use event::*;
mod lens;
pub use lens::*;
mod modifiers;
pub use modifiers::*;
mod view;
pub use view::*;
mod views;
pub use views::*;
mod viewid;
pub use viewid::*;
mod viewtuple;
pub use viewtuple::*;
mod winit_loop;
pub use winit_loop::vui;

pub use vello::{
    self, Scene,
    kurbo::{self, Point, Rect, Size, Vec2},
    peniko::{self, Color},
};

pub trait RunView {
    fn run(self);
}

impl<T> RunView for T
where
    T: View,
{
    fn run(self) {
        vui(self).expect("Expected no error");
    }
}
