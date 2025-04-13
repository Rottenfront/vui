use crate::*;
use std::any::Any;

/// Struct for the `background` modifier.
#[derive(Clone)]
pub struct Background<V, BG> {
    child: V,
    background: BG,
}

impl<V, BG> DynView for Background<V, BG>
where
    V: DynView,
    BG: DynView + Clone,
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
        path.push(1);
        self.background.process(event, path, ctx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        path.push(1);
        let mut scene = self.background.draw(path, ctx);
        path.pop();
        path.push(0);
        scene.append(&self.child.draw(path, ctx), None);
        path.pop();
        scene
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        path.push(0);
        let child_size = self.child.layout(path, args);
        path.pop();
        path.push(1);
        self.background
            .layout(path, &mut args.with_size(child_size));
        path.pop();
        child_size
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        path.push(1);
        let vid = self.background.hittest(path, pt, ctx);
        path.pop();
        vid
    }

    fn commands(&self, path: &mut IdPath, ctx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        self.child.commands(path, ctx, cmds);
        path.pop();
        path.push(1);
        self.background.commands(path, ctx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, ctx: &mut Context, map: &mut Vec<ViewId>) {
        path.push(0);
        self.child.gc(path, ctx, map);
        path.pop();
        path.push(1);
        self.background.gc(path, ctx, map);
        path.pop();
    }
}

impl<V, BG> Background<V, BG>
where
    V: View,
    BG: View + Clone,
{
    pub fn new(child: V, background: BG) -> Self {
        Self { child, background }
    }
}
