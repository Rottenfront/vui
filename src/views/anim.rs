use crate::*;
use std::any::Any;

#[derive(Clone)]
pub struct AnimView<V, F> {
    child: V,
    func: F,
}

impl<V, F> AnimView<V, F>
where
    V: View,
    F: Fn(&mut Context, f64) + 'static + Clone,
{
    pub fn new(child: V, func: F) -> Self {
        Self { child, func }
    }
}

impl<V, F> DynView for AnimView<V, F>
where
    V: View,
    F: Fn(&mut Context, f64) + 'static + Clone,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if let Event::Anim = event {
            (self.func)(ctx, 1.0 / 60.0) // XXX: assume 60fps for now.
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
        let sz = self.child.layout(path, args);
        path.pop();
        sz
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
        map.push(ctx.view_id(path));
        path.push(0);
        self.child.gc(path, ctx, map);
        path.pop();
    }
}
