use crate::*;

#[derive(Clone)]
pub struct Spacer {}

impl DynView for Spacer {
    fn draw(&self, _path: &mut IdPath, _ctx: &mut Context) -> Scene {
        Scene::new()
    }
    fn layout(&self, _path: &mut IdPath, _args: &mut LayoutArgs) -> Size {
        Size::ZERO
    }

    fn is_flexible(&self) -> bool {
        true
    }
}

/// Inserts a flexible space in a stack.
pub fn spacer() -> Spacer {
    Spacer {}
}
