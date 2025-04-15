use crate::*;
use std::any::Any;

#[derive(Clone)]
pub struct MapView<S1, SF, F> {
    value: S1,
    set_value: SF,
    func: F,
}

impl<S1, V, SF, F> DynView for MapView<S1, SF, F>
where
    V: View,
    S1: Clone + 'static,
    SF: Fn(S1, &mut Context) + Clone + 'static,
    F: Fn(StateHandle<S1>, &mut Context) -> V + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let id = ctx.view_id(path);
        ctx.set_state(id, self.value.clone());
        let s = StateHandle::new(id);
        path.push(0);
        (self.func)(s, ctx).process(event, path, ctx, actions);
        path.pop();

        // If processing the event changed the state, then call the set_value function.
        if ctx.is_dirty(id) {
            (self.set_value)(ctx[s].clone(), ctx)
        }
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        let id = ctx.view_id(path);
        ctx.set_state(id, self.value.clone());
        path.push(0);
        let scene = (self.func)(StateHandle::new(id), ctx).draw(path, ctx);
        path.pop();
        scene
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        let id = args.ctx.view_id(path);
        args.ctx.set_state(id, self.value.clone());

        path.push(0);
        let size = (self.func)(StateHandle::new(id), args.ctx).layout(path, args);
        path.pop();
        size
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        let id = ctx.view_id(path);
        ctx.set_state(id, self.value.clone());
        path.push(0);
        let hit_id = (self.func)(StateHandle::new(id), ctx).hittest(path, pt, ctx);
        path.pop();
        hit_id
    }

    fn commands(&self, path: &mut IdPath, ctx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let id = ctx.view_id(path);
        ctx.set_state(id, self.value.clone());
        path.push(0);
        (self.func)(StateHandle::new(id), ctx).commands(path, ctx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, ctx: &mut Context, map: &mut Vec<ViewId>) {
        let id = ctx.view_id(path);
        ctx.set_state(id, self.value.clone());
        map.push(id);
        path.push(0);
        (self.func)(StateHandle::new(id), ctx).gc(path, ctx, map);
        path.pop();
    }
}

/// Maps state into local state.
///
/// For example:
///
/// ```no_run
/// # use vui::*;
///
/// #[derive(Debug, Default)]
/// struct MyState {
///     x: f32,
/// }
///
/// fn main() {
///     vui(state(MyState::default, |state, ctx| {
///         vstack((
///             format!("value: {:?}", ctx[state]).padding(Auto),
///             map(
///                 ctx[state].x * 0.01,
///                 move |v, ctx| ctx[state].x = v * 100.0,
///                 |s, _| knob(s).padding(Auto),
///             ),
///         ))
///     }));
/// }
/// ```
pub fn map<S, SF, F>(value: S, set_value: SF, func: F) -> impl View
where
    MapView<S, SF, F>: view::View,
{
    MapView {
        value,
        set_value,
        func,
    }
}
