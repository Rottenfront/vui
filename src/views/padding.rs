use crate::*;
use kurbo::Affine;
use std::any::Any;

/// Struct for the `padding` modifier.
#[derive(Clone)]
pub struct Padding<V> {
    child: V,
    padding: [f64; 4],
}

impl<V> DynView for Padding<V>
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
        let off = Vec2::new(self.padding[0], self.padding[1]);
        path.push(0);
        self.child.process(&event.offset(-off), path, ctx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        let mut scene = Scene::new();
        let translate = Affine::translate((self.padding[0], self.padding[1]));
        path.push(0);
        scene.append(&self.child.draw(path, ctx), Some(translate));
        path.pop();
        scene
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        path.push(0);
        let child_size = self.child.layout(
            path,
            &mut args.with_size(
                args.size
                    - (
                        self.padding[0] + self.padding[2],
                        self.padding[1] + self.padding[3],
                    )
                        .into(),
            ),
        );
        path.pop();
        child_size
            + Size::new(
                self.padding[0] + self.padding[2],
                self.padding[1] + self.padding[3],
            )
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let hit_id =
            self.child
                .hittest(path, pt - Vec2::new(self.padding[0], self.padding[1]), ctx);
        path.pop();
        hit_id
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

pub enum PaddingParam {
    Auto,
    Px(f64),
    /// Indicates padding in X and Y symmetrically
    XY(f64, f64),
    Full([f64; 4]),
}
pub struct Auto;
impl From<Auto> for PaddingParam {
    fn from(_val: Auto) -> Self {
        PaddingParam::Auto
    }
}
impl From<f64> for PaddingParam {
    fn from(val: f64) -> Self {
        PaddingParam::Px(val)
    }
}

impl<V> Padding<V>
where
    V: View,
{
    pub fn new(child: V, param: PaddingParam) -> Self {
        Self {
            child,
            padding: match param {
                PaddingParam::Auto => [5.0, 5.0, 5.0, 5.0],
                PaddingParam::Px(px) => [px, px, px, px],
                PaddingParam::XY(x, y) => [x, y, x, y],
                PaddingParam::Full(padding) => padding,
            },
        }
    }
}
