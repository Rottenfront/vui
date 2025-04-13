use crate::*;
use std::any::Any;

pub struct TapInfo {
    pub pt: Point,
    pub button: Option<MouseButton>,
    pub state: TouchState,
}

pub trait TapFn: Clone {
    fn call(&self, ctx: &mut Context, tap_info: TapInfo, actions: &mut Vec<Box<dyn Any>>);
}

#[derive(Clone)]
pub struct TapFunc<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, TapInfo) -> A + Clone + 'static> TapFn for TapFunc<F> {
    fn call(&self, ctx: &mut Context, tap_info: TapInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new((self.f)(ctx, tap_info)))
    }
}

#[derive(Clone)]
pub struct TapPositionFunc<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, Point, Option<MouseButton>) -> A + Clone + 'static> TapFn
    for TapPositionFunc<F>
{
    fn call(&self, ctx: &mut Context, tap_info: TapInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new((self.f)(ctx, tap_info.pt, tap_info.button)))
    }
}

#[derive(Clone)]
pub struct TapAdapter<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context) -> A + Clone + 'static> TapFn for TapAdapter<F> {
    fn call(&self, ctx: &mut Context, _tap_info: TapInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new((self.f)(ctx)))
    }
}

#[derive(Clone)]
pub struct TapActionAdapter<A> {
    pub action: A,
}

impl<A: Clone + 'static> TapFn for TapActionAdapter<A> {
    fn call(&self, _ctx: &mut Context, _tap_info: TapInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new(self.action.clone()))
    }
}

/// Struct for the `tap` gesture.
#[derive(Clone)]
pub struct Tap<V, F> {
    /// Child view tree.
    child: V,

    /// Called when a tap occurs.
    func: F,
}

impl<V, F> Tap<V, F>
where
    V: View,
    F: TapFn + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> DynView for Tap<V, F>
where
    V: View,
    F: TapFn + 'static,
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
                }
            }
            Event::TouchEnd { id, position } => {
                if ctx.touches[*id] == vid {
                    ctx.touches[*id] = ViewId::default();
                    self.func.call(
                        ctx,
                        TapInfo {
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
