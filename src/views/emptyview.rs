use crate::*;

#[derive(Clone)]
pub struct EmptyView {}

impl DynView for EmptyView {
    fn draw(&self, _path: &mut IdPath, _ctx: &mut Context) -> Scene {
        Scene::new()
    }
    fn layout(&self, _path: &mut IdPath, _args: &mut LayoutArgs) -> Size {
        Size::ZERO
    }
}
