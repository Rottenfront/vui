use crate::*;
use std::any::Any;

/// Weak reference to app state.
///
/// To get the underlying value, you'll need a `Context`, which is passed
/// to all event handlers, and functions passed to `state`.
pub struct StateHandle<S> {
    pub(crate) id: ViewId,
    phantom: std::marker::PhantomData<S>,
}

impl<S> Copy for StateHandle<S> {}

impl<S> Clone for StateHandle<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: 'static> StateHandle<S> {
    pub fn new(id: ViewId) -> Self {
        Self {
            id,
            phantom: Default::default(),
        }
    }

    /// Makes it convenient to get a function to set the value.
    pub fn setter(self) -> impl Fn(S, &mut Context) {
        move |s, ctx| ctx[self] = s
    }
}

impl<S: 'static> Binding<S> for StateHandle<S> {
    fn get<'a>(&self, ctx: &'a Context) -> &'a S {
        ctx.get(*self)
    }
    fn get_mut<'a>(&self, ctx: &'a mut Context) -> &'a mut S {
        ctx.get_mut(*self)
    }
}

#[derive(Clone)]
pub struct StateView<D, F> {
    default: D,
    func: F,
}

impl<S, V, D, F> DynView for StateView<D, F>
where
    V: View,
    S: 'static,
    D: Fn() -> S + Clone + 'static,
    F: Fn(StateHandle<S>, &Context) -> V + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let id = ctx.view_id(path);
        ctx.init_state(id, &self.default);
        path.push(0);
        (self.func)(StateHandle::new(id), ctx).process(event, path, ctx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        let id = ctx.view_id(path);
        ctx.init_state(id, &self.default);
        path.push(0);
        let scene = (self.func)(StateHandle::new(id), ctx).draw(path, ctx);
        path.pop();
        scene
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        let id = args.ctx.view_id(path);
        args.ctx.init_state(id, &self.default);

        // Do we need to recompute layout?
        let mut compute_layout = true;

        if let Some(deps) = args.ctx.deps.get(&id) {
            let mut any_dirty = false;
            for dep in deps {
                if let Some(holder) = args.ctx.state_map.get_mut(dep) {
                    if holder.dirty {
                        any_dirty = true;
                        break;
                    }
                }
            }

            compute_layout = any_dirty;
        }

        if compute_layout {
            args.ctx.id_stack.push(id);

            let view = (self.func)(StateHandle::new(id), args.ctx);

            path.push(0);
            let child_size = view.layout(path, args);

            // Compute layout dependencies.
            let mut deps = vec![];
            deps.append(&mut args.ctx.id_stack.clone());
            view.gc(path, args.ctx, &mut deps);

            path.pop();

            args.ctx.deps.insert(id, deps);

            let layout_box = LayoutBox {
                rect: Rect::from_origin_size(Point::ZERO, child_size),
                offset: Vec2::ZERO,
            };
            args.ctx.update_layout(path, layout_box);

            args.ctx.id_stack.pop();
        }

        args.ctx.get_layout(path).rect.size()
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        let id = ctx.view_id(path);
        ctx.init_state(id, &self.default);
        path.push(0);
        let hit_id = (self.func)(StateHandle::new(id), ctx).hittest(path, pt, ctx);
        path.pop();
        hit_id
    }

    fn commands(&self, path: &mut IdPath, ctx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let id = ctx.view_id(path);
        ctx.init_state(id, &self.default);
        path.push(0);
        (self.func)(StateHandle::new(id), ctx).commands(path, ctx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, ctx: &mut Context, map: &mut Vec<ViewId>) {
        let id = ctx.view_id(path);
        ctx.init_state(id, &self.default);
        map.push(id);
        path.push(0);
        (self.func)(StateHandle::new(id), ctx).gc(path, ctx, map);
        path.pop();
    }
}

/// State allows you to associate some state with a view.
/// This is what you'll use for a data model, as well as per-view state.
/// Your state should be efficiently clonable. Use Rc as necessary.
///
/// `initial` is the initial value for your state.
///
/// `f` callback which is passed a `State<S>`
pub fn state<
    S: 'static,
    V: View,
    D: Fn() -> S + Clone + 'static,
    F: Fn(StateHandle<S>, &Context) -> V + Clone + 'static,
>(
    initial: D,
    f: F,
) -> StateView<D, F> {
    StateView {
        default: initial,
        func: f,
    }
}

/// Convenience to get the context.
pub fn with_ctx<V: View, F: Fn(&Context) -> V + Clone + 'static>(f: F) -> impl View {
    state(|| (), move |_, ctx| f(ctx))
}

/// Convenience to retreive a reference to a value in the context.
pub fn with_ref<V: View, F: Fn(&T) -> V + Clone + 'static, T>(
    binding: impl Binding<T>,
    f: F,
) -> impl View {
    with_ctx(move |ctx| f(binding.get(ctx)))
}
