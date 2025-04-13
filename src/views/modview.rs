use crate::*;
use std::any::Any;
use std::fmt;

#[derive(Clone)]
pub struct ModView<S, F> {
    pub func: F,
    pub value: S,
}

impl<S, F> fmt::Display for ModView<S, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ModView")
    }
}

impl<S, V, F> DynView for ModView<S, F>
where
    V: View,
    S: Clone + Default + 'static,
    F: Fn(S, &mut Context) -> V + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        path.push(0);
        (self.func)(self.value.clone(), ctx).process(event, path, ctx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        path.push(0);
        let scene = (self.func)(self.value.clone(), ctx).draw(path, ctx);
        path.pop();
        scene
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        path.push(0);
        let size = (self.func)(self.value.clone(), args.ctx).layout(path, args);
        path.pop();
        size
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let hit_id = (self.func)(self.value.clone(), ctx).hittest(path, pt, ctx);
        path.pop();
        hit_id
    }

    fn commands(&self, path: &mut IdPath, ctx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        (self.func)(self.value.clone(), ctx).commands(path, ctx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, ctx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(ctx.view_id(path));
        path.push(0);
        (self.func)(self.value.clone(), ctx).gc(path, ctx, map);
        path.pop();
    }
}

/// Passes a value to a function. Value can be updated by modifiers.
pub fn modview<
    S: Clone + Default + 'static,
    V: View,
    F: Fn(S, &mut Context) -> V + Clone + 'static,
>(
    f: F,
) -> ModView<S, F> {
    ModView {
        func: f,
        value: Default::default(),
    }
}
