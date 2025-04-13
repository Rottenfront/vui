use crate::*;
use std::{any::Any, sync::Arc};

/// Struct for the `window_title` modifier.
#[derive(Clone)]
pub struct TitleView<V> {
    child: V,
    title: Arc<str>,
}

impl<V> TitleView<V>
where
    V: View,
{
    pub fn new(v: V, title: &str) -> Self {
        Self {
            child: v,
            title: title.into(),
        }
    }
}

impl<V> DynView for TitleView<V>
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
        self.child.process(event, path, ctx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        path.push(0);
        let scene = self.child.draw(path, ctx);
        path.pop();
        if ctx.window_title != self.title {
            ctx.window_title = self.title.clone();
        }
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

/// Struct for the `fullscreen` modifier.
#[derive(Clone)]
pub struct FullscreenView<V> {
    child: V,
}

impl<V> FullscreenView<V>
where
    V: View,
{
    pub fn new(v: V) -> Self {
        Self { child: v }
    }
}

impl<V> DynView for FullscreenView<V>
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
        self.child.process(event, path, ctx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        path.push(0);
        let scene = self.child.draw(path, ctx);
        path.pop();
        ctx.fullscreen = true;
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
