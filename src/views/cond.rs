use crate::*;
use std::any::Any;

/// Struct for `cond`
#[derive(Clone)]
pub struct Cond<V0, V1> {
    cond: bool,
    if_true: V0,
    if_false: V1,
}

impl<V0, V1> DynView for Cond<V0, V1>
where
    V0: DynView,
    V1: DynView,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if self.cond {
            path.push(0);
            self.if_true.process(event, path, ctx, actions);
            path.pop();
        } else {
            path.push(1);
            self.if_false.process(event, path, ctx, actions);
            path.pop();
        }
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        if self.cond {
            path.push(0);
            let scene = self.if_true.draw(path, ctx);
            path.pop();
            scene
        } else {
            path.push(1);
            let scene = self.if_false.draw(path, ctx);
            path.pop();
            scene
        }
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        if self.cond {
            path.push(0);
            let size = self.if_true.layout(path, args);
            path.pop();
            size
        } else {
            path.push(1);
            let size = self.if_false.layout(path, args);
            path.pop();
            size
        }
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        if self.cond {
            path.push(0);
            let id = self.if_true.hittest(path, pt, ctx);
            path.pop();
            id
        } else {
            path.push(1);
            let id = self.if_false.hittest(path, pt, ctx);
            path.pop();
            id
        }
    }

    fn commands(&self, path: &mut IdPath, ctx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        if self.cond {
            path.push(0);
            self.if_true.commands(path, ctx, cmds);
            path.pop();
        } else {
            path.push(1);
            self.if_false.commands(path, ctx, cmds);
            path.pop();
        }
    }

    fn gc(&self, path: &mut IdPath, ctx: &mut Context, map: &mut Vec<ViewId>) {
        if self.cond {
            path.push(0);
            self.if_true.gc(path, ctx, map);
            path.pop();
        } else {
            path.push(1);
            self.if_false.gc(path, ctx, map);
            path.pop();
        }
    }
}

/// Switches between views according to a boolean.
pub fn cond(cond: bool, if_true: impl View, if_false: impl View) -> impl View {
    Cond {
        cond,
        if_true,
        if_false,
    }
}
