use crate::*;
use kurbo::Affine;
use peniko::Mix;
use std::any::Any;

#[derive(Clone)]
pub struct Clip<V> {
    child: V,
}

impl<V> Clip<V>
where
    V: DynView,
{
    fn geom(&self, path: &IdPath, ctx: &mut Context) -> Rect {
        ctx.get_layout(path).rect
    }

    pub fn new(child: V) -> Self {
        Self { child }
    }
}

impl<V> DynView for Clip<V>
where
    V: DynView,
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
        let rect = self.geom(path, ctx);
        let mut scene = Scene::new();
        scene.push_layer(Mix::Clip, 1.0, Affine::IDENTITY, &rect);
        path.push(0);
        scene.append(&self.child.draw(path, ctx), None);
        path.pop();
        scene.pop_layer();
        scene
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        path.push(0);
        self.child.layout(path, args);
        path.pop();
        args.ctx.update_layout(
            path,
            LayoutBox {
                rect: Rect::from_origin_size(Point::ZERO, args.size),
                offset: Vec2::ZERO,
            },
        );
        // XXX: should this expand to the available space?
        args.size
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        let rect = self.geom(path, ctx);

        if rect.contains(pt) {
            // Test against children.
            path.push(0);
            let vid = self.child.hittest(path, pt, ctx);
            path.pop();
            vid
        } else {
            None
        }
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
