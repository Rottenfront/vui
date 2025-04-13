use crate::*;
use std::any::Any;

#[derive(Clone, PartialEq)]
pub enum TouchState {
    Begin,
    End,
}

#[derive(Clone)]
pub struct TouchInfo {
    /// The position of the touch in local space of the view that received the touch.
    pub pt: Point,

    /// The mouse button that was used for the touch if a mouse was used.
    pub button: Option<MouseButton>,

    /// The state of the touch. IE: Begin or End.
    pub state: TouchState,
}

pub trait TouchFn: Clone {
    fn call(&self, ctx: &mut Context, touch_info: TouchInfo, actions: &mut Vec<Box<dyn Any>>);
}

#[derive(Clone)]
pub struct TouchFunc<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, TouchInfo) -> A + Clone> TouchFn for TouchFunc<F> {
    fn call(&self, ctx: &mut Context, touch_info: TouchInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new((self.f)(ctx, touch_info)))
    }
}

#[derive(Clone)]
pub struct TouchPositionFunc<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, Point, Option<MouseButton>) -> A + Clone> TouchFn
    for TouchPositionFunc<F>
{
    fn call(&self, ctx: &mut Context, touch_info: TouchInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new((self.f)(ctx, touch_info.pt, touch_info.button)))
    }
}

#[derive(Clone)]
pub struct TouchAdapter<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context) -> A + Clone> TouchFn for TouchAdapter<F> {
    fn call(&self, ctx: &mut Context, _touch_info: TouchInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new((self.f)(ctx)))
    }
}

#[derive(Clone)]
pub struct TouchActionAdapter<A> {
    pub action: A,
}

impl<A: Clone + 'static> TouchFn for TouchActionAdapter<A> {
    fn call(&self, _ctx: &mut Context, _touch_info: TouchInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new(self.action.clone()))
    }
}

/// Struct for the `touch` gesture.
#[derive(Clone)]
pub struct Touch<V: View, F> {
    /// Child view tree.
    child: V,

    /// Called when a touch occurs.
    func: F,
}

impl<V, F> Touch<V, F>
where
    V: View,
    F: TouchFn + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> DynView for Touch<V, F>
where
    V: View,
    F: TouchFn + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let vid = ctx.view_id(path);
        match &event {
            Event::TouchBegin { id, position } => {
                if self.hittest(path, *position, ctx).is_some() {
                    ctx.touches[*id] = vid;
                    self.func.call(
                        ctx,
                        TouchInfo {
                            pt: *position,
                            button: ctx.mouse_button,
                            state: TouchState::Begin,
                        },
                        actions,
                    )
                }
            }
            Event::TouchEnd { id, position } => {
                if ctx.touches[*id] == vid {
                    ctx.touches[*id] = ViewId::default();
                    self.func.call(
                        ctx,
                        TouchInfo {
                            pt: *position,
                            button: ctx.mouse_button,
                            state: TouchState::End,
                        },
                        actions,
                    )
                }
            }
            _ => (),
        }
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        path.push(0);
        let scene = self.child.draw(path, ctx);
        path.pop();
        scene
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        path.push(0);
        let size = self.child.layout(path, args);
        path.pop();
        size
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let id = self.child.hittest(path, pt, ctx);
        path.pop();
        id
    }

    fn commands(&self, path: &mut IdPath, ctx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        self.child.commands(path, ctx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, ctx: &mut Context, map: &mut Vec<ViewId>) {
        path.push(0);
        self.child.gc(path, ctx, map);
        path.pop();
    }
}
