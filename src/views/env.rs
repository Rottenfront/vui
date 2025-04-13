use crate::*;
use std::any::Any;

#[derive(Clone)]
struct EnvView<S, V, F> {
    func: F,
    phantom_s: std::marker::PhantomData<S>,
    phantom_v: std::marker::PhantomData<V>,
}

impl<S, V, F> DynView for EnvView<S, V, F>
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
        (self.func)(ctx.init_env(&S::default), ctx).process(event, path, ctx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        path.push(0);
        let scene = (self.func)(ctx.init_env(&S::default), ctx).draw(path, ctx);
        path.pop();
        scene
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        path.push(0);
        let size = (self.func)(args.ctx.init_env(&S::default), args.ctx).layout(path, args);
        path.pop();
        size
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let vid = (self.func)(ctx.init_env(&S::default), ctx).hittest(path, pt, ctx);
        path.pop();
        vid
    }

    fn commands(&self, path: &mut IdPath, ctx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        (self.func)(ctx.init_env(&S::default), ctx).commands(path, ctx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, ctx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(ctx.view_id(path));
        path.push(0);
        (self.func)(ctx.init_env(&S::default), ctx).gc(path, ctx, map);
        path.pop();
    }
}

/// Reads from the environment.
pub fn env<S: Clone + Default + 'static, V: View, F: Fn(S, &mut Context) -> V + Clone + 'static>(
    f: F,
) -> impl View {
    EnvView {
        func: f,
        phantom_s: Default::default(),
        phantom_v: Default::default(),
    }
}

/// Struct for the `env` modifier.
#[derive(Clone)]
pub struct SetenvView<V, E> {
    child: V,
    env_val: E,
}

impl<V, E> SetenvView<V, E>
where
    V: View,
    E: Clone + 'static,
{
    pub fn new(child: V, env_val: E) -> Self {
        Self { child, env_val }
    }
}

impl<V, E> DynView for SetenvView<V, E>
where
    V: View,
    E: Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let old = ctx.set_env(&self.env_val);
        path.push(0);
        self.child.process(event, path, ctx, actions);
        path.pop();
        old.and_then(|s| ctx.set_env(&s));
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        let old = ctx.set_env(&self.env_val);
        path.push(0);
        let scene = self.child.draw(path, ctx);
        path.pop();
        old.and_then(|s| ctx.set_env(&s));
        scene
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        let old = args.ctx.set_env(&self.env_val);
        path.push(0);
        let size = self.child.layout(path, args);
        path.pop();
        old.and_then(|s| args.ctx.set_env(&s));
        size
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        let old = ctx.set_env(&self.env_val);
        path.push(0);
        let r = self.child.hittest(path, pt, ctx);
        path.pop();
        old.and_then(|s| ctx.set_env(&s));
        r
    }

    fn commands(&self, path: &mut IdPath, ctx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let old = ctx.set_env(&self.env_val);
        path.push(0);
        self.child.commands(path, ctx, cmds);
        path.pop();
        old.and_then(|s| ctx.set_env(&s));
    }

    fn gc(&self, path: &mut IdPath, ctx: &mut Context, map: &mut Vec<ViewId>) {
        let old = ctx.set_env(&self.env_val);
        path.push(0);
        self.child.gc(path, ctx, map);
        path.pop();
        old.and_then(|s| ctx.set_env(&s));
    }
}
