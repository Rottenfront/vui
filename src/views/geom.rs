use crate::*;
use std::any::Any;

/// Struct for the `geom` modifier.
#[derive(Clone)]
pub struct Geom<V, F> {
    child: V,
    func: F,
}

impl<V, F> DynView for Geom<V, F>
where
    V: View,
    F: Fn(&mut Context, Size) + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        path.push(0);
        self.child.process(event, path, ctx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        let rect = ctx.get_layout(path).rect;
        (self.func)(ctx, rect.size());
        path.push(0);
        let scene = self.child.draw(path, ctx);
        path.pop();
        scene
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        path.push(0);
        let size = self.child.layout(path, args);
        path.pop();

        args.ctx.update_layout(
            path,
            LayoutBox {
                rect: Rect::from_center_size(Point::ZERO, size),
                offset: Vec2::ZERO,
            },
        );

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
        map.push(ctx.view_id(path));
        path.push(0);
        self.child.gc(path, ctx, map);
        path.pop();
    }
}

impl<V, F> Geom<V, F>
where
    V: View,
    F: Fn(&mut Context, Size) + Clone + 'static,
{
    pub fn new(child: V, f: F) -> Self {
        Self { child, func: f }
    }
}
