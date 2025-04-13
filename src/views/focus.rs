use crate::*;
use std::any::Any;

/// Struct for the `focus` modifier.
#[derive(Clone)]
pub struct Focus<F> {
    func: F,
}

impl<V, F> DynView for Focus<F>
where
    V: View,
    F: Fn(bool) -> V + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let vid = ctx.view_id(path);
        match &event {
            Event::TouchBegin { id: _, position } => {
                if self.hittest(path, *position, ctx).is_some() {
                    ctx.focused_id = Some(vid);
                    ctx.set_dirty();
                }
            }
            Event::Key(Key::Escape) => {
                if ctx.focused_id == Some(vid) {
                    ctx.focused_id = None;
                    ctx.set_dirty();
                }
            }
            _ => (),
        }
        path.push(0);
        (self.func)(Some(vid) == ctx.focused_id).process(event, path, ctx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        let id = ctx.view_id(path);
        path.push(0);
        let scene = (self.func)(Some(id) == ctx.focused_id).draw(path, ctx);
        path.pop();
        scene
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        let id = args.ctx.view_id(path);
        path.push(0);
        let size = (self.func)(Some(id) == args.ctx.focused_id).layout(path, args);
        path.pop();
        size
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        let id = ctx.view_id(path);
        path.push(0);
        let vid = (self.func)(Some(id) == ctx.focused_id).hittest(path, pt, ctx);
        path.pop();
        vid
    }

    fn commands(&self, path: &mut IdPath, ctx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let id = ctx.view_id(path);
        path.push(0);
        (self.func)(Some(id) == ctx.focused_id).commands(path, ctx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, ctx: &mut Context, map: &mut Vec<ViewId>) {
        let id = ctx.view_id(path);
        path.push(0);
        (self.func)(Some(id) == ctx.focused_id).gc(path, ctx, map);
        path.pop();
    }
}

/// Calls calls a function with true if the view subtree returned
/// by the function has the keyboard focus.
pub fn focus<V: View, F: Fn(bool) -> V + Clone + 'static>(f: F) -> impl View {
    Focus { func: f }
}
