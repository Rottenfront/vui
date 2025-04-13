use crate::*;
use std::any::{Any, TypeId};
use vello::Scene;

pub struct LayoutArgs<'a> {
    pub size: Size,
    pub ctx: &'a mut Context,
}

impl<'a> LayoutArgs<'a> {
    pub fn with_size(&mut self, size: Size) -> LayoutArgs {
        LayoutArgs {
            size,
            ctx: self.ctx,
        }
    }
}

/// Object-safe part of View for compatibility with AnyView.
pub trait DynView: 'static {
    /// Accumulates information about menu bar commands.
    fn commands(&self, _path: &mut IdPath, _ctx: &mut Context, _cmds: &mut Vec<CommandInfo>) {}

    /// Draws the view.
    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene;

    /// Gets IDs for views currently in use.
    ///
    /// Push onto map if the view stores layout or state info.
    fn gc(&self, _path: &mut IdPath, _ctx: &mut Context, _map: &mut Vec<ViewId>) {}

    /// Returns the topmost view which the point intersects.
    fn hittest(&self, _path: &mut IdPath, _pt: Point, _ctx: &mut Context) -> Option<ViewId> {
        None
    }

    /// For detecting flexible sized things in stacks.
    fn is_flexible(&self) -> bool {
        false
    }

    /// Lays out subviews and return the size of the view.
    ///
    /// `size` is the available size for the view
    ///
    /// Note that we should probably have a separate text
    /// sizing interface so we don't need a GPU and graphics
    /// context set up to test layout.
    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size;

    /// Processes an event.
    fn process(
        &self,
        _event: &Event,
        _path: &mut IdPath,
        _ctx: &mut Context,
        _actions: &mut Vec<Box<dyn Any>>,
    ) {
    }

    /// Returns the type ID of the underlying view.
    fn tid(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

pub trait View: DynView + Clone {}

impl<T: DynView + Clone> View for T {}
