use crate::*;
use std::any::Any;

pub trait HoverFn: Clone {
    fn call(&self, ctx: &mut Context, pt: Point, inside: bool, actions: &mut Vec<Box<dyn Any>>);
}

#[derive(Clone)]
pub struct HoverFuncP<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, Point) -> A + Clone + 'static> HoverFn for HoverFuncP<F> {
    fn call(&self, ctx: &mut Context, pt: Point, inside: bool, actions: &mut Vec<Box<dyn Any>>) {
        if inside {
            actions.push(Box::new((self.f)(ctx, pt)))
        }
    }
}

#[derive(Clone)]
pub struct HoverFunc<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, bool) -> A + Clone + 'static> HoverFn for HoverFunc<F> {
    fn call(&self, ctx: &mut Context, _pt: Point, inside: bool, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new((self.f)(ctx, inside)))
    }
}

/// Struct for the `hover` and 'hover_p` gestures.
#[derive(Clone)]
pub struct Hover<V, F> {
    child: V,
    func: F,
}

impl<V, F> Hover<V, F>
where
    V: View,
    F: HoverFn + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> DynView for Hover<V, F>
where
    V: View,
    F: HoverFn + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if let Event::TouchMove { position, .. } = &event {
            if ctx.mouse_button.is_none() {
                let inside = self.hittest(path, *position, ctx).is_some();
                self.func.call(ctx, *position, inside, actions);
            }
        }
        if let Event::TouchEnd { position, .. } = &event {
            let inside = self.hittest(path, *position, ctx).is_some();
            self.func.call(ctx, *position, inside, actions);
        }
        path.push(0);
        self.child.process(event, path, ctx, actions);
        path.pop();
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
