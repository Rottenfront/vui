use crate::*;
use std::any::Any;
use std::any::TypeId;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Struct for `any_view`
#[derive(Clone)]
pub struct AnyView {
    child: Box<dyn View>,
}

impl AnyView {
    pub fn new(child: impl View) -> Self {
        Self {
            child: Box::new(child),
        }
    }

    fn id(&self) -> TypeId {
        self.child.tid()
    }

    fn id_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.id().hash(&mut hasher);
        hasher.finish()
    }
}

impl View for AnyView {
    fn tid(&self) -> TypeId {
        self.child.tid()
    }

    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        path.push(self.id_hash());
        self.child.process(event, path, ctx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        path.push(self.id_hash());
        let scene = self.child.draw(path, ctx);
        path.pop();
        scene
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        path.push(self.id_hash());
        let size = self.child.layout(path, args);
        path.pop();
        size
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        path.push(self.id_hash());
        let vid = self.child.hittest(path, pt, ctx);
        path.pop();
        vid
    }

    fn commands(&self, path: &mut IdPath, ctx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(self.id_hash());
        self.child.commands(path, ctx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, ctx: &mut Context, map: &mut Vec<ViewId>) {
        path.push(self.id_hash());
        self.child.gc(path, ctx, map);
        path.pop();
    }
}

/// Switches between views according to a boolean.
pub fn any_view(view: impl View) -> AnyView {
    AnyView {
        child: Box::new(view),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_typeid() {
        let b: Box<dyn View> = Box::new(EmptyView {});
        let tid = b.tid();
        println!("{:?}", tid);
        assert_eq!(tid, TypeId::of::<EmptyView>());
    }

    #[test]
    fn test_typeid2() {
        let a = EmptyView {};
        let b = rectangle();
        assert_ne!(a.tid(), b.tid());
    }

    #[test]
    fn test_typeid3() {
        let a = any_view(EmptyView {});
        let b = any_view(rectangle());
        assert_ne!(a.tid(), b.tid());
    }
}
