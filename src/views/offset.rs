use crate::*;
use kurbo::Affine;
use std::any::Any;

/// Struct for the `offset` modifier.
#[derive(Clone)]
pub struct Offset<V> {
    child: V,
    offset: Vec2,
}

impl<V> DynView for Offset<V>
where
    V: View,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        path.push(0);
        self.child
            .process(&event.offset(-self.offset), path, ctx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        let mut scene = Scene::new();
        let translate = Affine::translate(self.offset);
        path.push(0);
        scene.append(&self.child.draw(path, ctx), Some(translate));
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
        let hit_id = self.child.hittest(path, pt - self.offset, ctx);
        path.pop();
        hit_id
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

impl<V> Offset<V>
where
    V: View,
{
    pub fn new(child: V, offset: Vec2) -> Self {
        Self { child, offset }
    }
}
