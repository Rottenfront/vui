use crate::*;

/// Struct for `canvas`
#[derive(Clone)]
pub struct Canvas<F> {
    func: F,
}

impl<F> DynView for Canvas<F>
where
    F: Fn(&mut Context, Rect) -> Scene + Clone + 'static,
{
    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        let rect = ctx.get_layout(path).rect;

        (self.func)(ctx, rect)
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        args.ctx.update_layout(
            path,
            LayoutBox {
                rect: Rect::from_origin_size(Point::ZERO, args.size),
                offset: Vec2::ZERO,
            },
        );
        args.size
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        let rect = ctx.get_layout(path).rect;

        if rect.contains(pt) {
            Some(ctx.view_id(path))
        } else {
            None
        }
    }

    fn gc(&self, path: &mut IdPath, ctx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(ctx.view_id(path));
    }
}

/// Canvas for GPU drawing with Vger. See https://github.com/audulus/vger-rs.
pub fn canvas<F: Fn(&mut Context, Rect) -> Scene + Clone + 'static>(f: F) -> Canvas<F> {
    Canvas { func: f }
}
