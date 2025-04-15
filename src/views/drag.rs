use crate::*;
use std::any::Any;
use std::marker::PhantomData;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum GestureState {
    Began,
    Changed,
    Ended,
}

pub trait DragFn: Clone {
    fn call(
        &self,
        ctx: &mut Context,
        pt: Point,
        delta: Vec2,
        state: GestureState,
        button: Option<MouseButton>,
        actions: &mut Vec<Box<dyn Any>>,
    );
}

#[derive(Clone)]
pub struct DragFunc<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, Vec2, GestureState, Option<MouseButton>) -> A + Clone> DragFn
    for DragFunc<F>
{
    fn call(
        &self,
        ctx: &mut Context,
        _pt: Point,
        delta: Vec2,
        state: GestureState,
        button: Option<MouseButton>,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        actions.push(Box::new((self.f)(ctx, delta, state, button)))
    }
}

#[derive(Clone)]
pub struct DragFuncP<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, Point, GestureState, Option<MouseButton>) -> A + Clone> DragFn
    for DragFuncP<F>
{
    fn call(
        &self,
        ctx: &mut Context,
        pt: Point,
        _delta: Vec2,
        state: GestureState,
        button: Option<MouseButton>,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        actions.push(Box::new((self.f)(ctx, pt, state, button)))
    }
}

#[derive(Clone)]
pub struct DragFuncS<F, B, T> {
    pub f: F,
    pub b: B,
    pub phantom: PhantomData<T>,
}

impl<
    F: Fn(&mut T, Vec2, GestureState, Option<MouseButton>) -> A + Clone + 'static,
    B: Binding<T>,
    A: 'static,
    T: Clone + 'static,
> DragFn for DragFuncS<F, B, T>
{
    fn call(
        &self,
        ctx: &mut Context,
        _pt: Point,
        delta: Vec2,
        state: GestureState,
        button: Option<MouseButton>,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        actions.push(Box::new((self.f)(
            self.b.get_mut(ctx),
            delta,
            state,
            button,
        )))
    }
}

/// Struct for the `drag` and `drag_p` gestures.
#[derive(Clone)]
pub struct Drag<V, F> {
    child: V,
    func: F,
    grab: bool,
}

impl<V, F> Drag<V, F>
where
    V: View,
    F: DragFn + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self {
            child: v,
            func: f,
            grab: false,
        }
    }

    pub fn grab_cursor(self) -> Self {
        Self {
            child: self.child,
            func: self.func,
            grab: true,
        }
    }
}

impl<V, F> DynView for Drag<V, F>
where
    V: View,
    F: DragFn + 'static,
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
            Event::TouchBegin { id, position } => {
                if ctx.touches[*id].is_default() && self.hittest(path, *position, ctx).is_some() {
                    ctx.touches[*id] = vid;
                    ctx.starts[*id] = *position;
                    ctx.previous_position[*id] = *position;
                    ctx.grab_cursor = self.grab;

                    self.func.call(
                        ctx,
                        *position,
                        Vec2::ZERO,
                        GestureState::Began,
                        ctx.mouse_button,
                        actions,
                    );
                }
            }
            Event::TouchMove {
                id,
                position,
                delta,
            } => {
                if ctx.touches[*id] == vid {
                    self.func.call(
                        ctx,
                        *position,
                        *delta,
                        GestureState::Changed,
                        ctx.mouse_button,
                        actions,
                    );
                    ctx.previous_position[*id] = *position;
                }
            }
            Event::TouchEnd { id, position } => {
                if ctx.touches[*id] == vid {
                    ctx.touches[*id] = ViewId::default();
                    ctx.grab_cursor = false;

                    self.func.call(
                        ctx,
                        *position,
                        Vec2::ZERO,
                        GestureState::Ended,
                        ctx.mouse_button,
                        actions,
                    );
                }
            }
            _ => {
                path.push(0);
                self.child.process(event, path, ctx, actions);
                path.pop();
            }
        }
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        path.push(0);
        let scene = self.child.draw(path, ctx);
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_drag() {
        let mut ctx = Context::new();

        let ui = state(
            || vec![],
            |states, _| {
                rectangle()
                    .color(CLEAR_COLOR)
                    .drag(move |ctx, _delta, state, _| ctx[states].push(state))
            },
        );
        let size = (100.0, 100.0).into();
        let mut path = vec![0];

        let rect_size = ui.layout(
            &mut path,
            &mut LayoutArgs {
                size,
                ctx: &mut ctx,
            },
        );
        assert_eq!(path.len(), 1);

        assert_eq!(rect_size, size);
        let s = StateHandle::<Vec<GestureState>>::new(ctx.view_id(&path));
        assert_eq!(ctx[s], vec![]);

        let events = [
            Event::TouchBegin {
                id: 0,
                position: (50.0, 50.0).into(),
            },
            Event::TouchMove {
                id: 0,
                position: (60.0, 50.0).into(),
                delta: (10.0, 0.0).into(),
            },
            Event::TouchEnd {
                id: 0,
                position: (60.0, 50.0).into(),
            },
        ];

        let mut actions = vec![];
        for event in &events {
            ui.process(event, &mut path, &mut ctx, &mut actions);
        }
        assert_eq!(path.len(), 1);

        assert_eq!(
            ctx[s],
            vec![
                GestureState::Began,
                GestureState::Changed,
                GestureState::Ended
            ]
        );
    }
}
