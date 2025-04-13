use crate::*;
use std::any::Any;

/// Describes if the KeyView action should trigger when pressing or releasing a key
#[derive(Clone)]
pub enum KeyViewKind {
    Pressed,
    Released,
}

/// Struct for the `key` modifier.
#[derive(Clone)]
pub struct KeyView<V, F> {
    child: V,
    func: F,
    kind: KeyViewKind,
}

impl<V, F, A> KeyView<V, F>
where
    V: View,
    F: Fn(&mut Context, Key) -> A + Clone + 'static,
{
    pub fn new_pressed(v: V, f: F) -> Self {
        KeyView {
            child: v,
            func: f,
            kind: KeyViewKind::Pressed,
        }
    }

    pub fn new_released(v: V, f: F) -> Self {
        KeyView {
            child: v,
            func: f,
            kind: KeyViewKind::Released,
        }
    }
}

impl<V, F, A> DynView for KeyView<V, F>
where
    V: View,
    F: Fn(&mut Context, Key) -> A + Clone + 'static,
    A: 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        match self.kind {
            KeyViewKind::Pressed => {
                if let Event::Key(key) = &event {
                    actions.push(Box::new((self.func)(ctx, *key)));
                } else {
                    path.push(0);
                    self.child.process(event, path, ctx, actions);
                    path.pop();
                }
            }
            KeyViewKind::Released => {
                if let Event::KeyReleased(key) = &event {
                    actions.push(Box::new((self.func)(ctx, *key)));
                } else {
                    path.push(0);
                    self.child.process(event, path, ctx, actions);
                    path.pop();
                }
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
