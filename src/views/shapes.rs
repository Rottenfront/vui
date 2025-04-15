use crate::*;

use peniko::Brush;
use vello::kurbo::{Affine, RoundedRect};

/// Struct for `circle`.
#[derive(Clone)]
pub struct Circle {
    paint: Brush,
}

impl Circle {
    fn geom(&self, path: &IdPath, ctx: &mut Context) -> (Point, f64) {
        let rect = ctx.get_layout(path).rect;

        (rect.center(), rect.width().min(rect.height()) / 2.0)
    }

    pub fn color(self, color: Color) -> Circle {
        Circle {
            paint: color.into(),
        }
    }
}

impl DynView for Circle {
    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        let (center, radius) = self.geom(path, ctx);
        let mut scene = Scene::new();
        scene.fill(
            peniko::Fill::EvenOdd,
            Affine::IDENTITY,
            &self.paint,
            None,
            &kurbo::Circle::new(center, radius),
        );
        scene
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
        let (center, radius) = self.geom(path, ctx);

        if pt.distance(center) < radius {
            Some(ctx.view_id(path))
        } else {
            None
        }
    }

    fn gc(&self, path: &mut IdPath, ctx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(ctx.view_id(path));
    }
}

/// Renders a circle which expands to fill available space.
pub fn circle() -> Circle {
    Circle {
        paint: AZURE_HIGHLIGHT.into(),
    }
}

/// Struct for `rectangle`.
#[derive(Clone)]
pub struct Rectangle {
    corner_radius: f64,
    paint: Brush,
}

impl Rectangle {
    fn geom(&self, path: &IdPath, ctx: &mut Context) -> Rect {
        ctx.get_layout(path).rect
    }

    /// Sets the fill color for the rectangle.
    pub fn color(self, color: Color) -> Rectangle {
        Rectangle {
            corner_radius: self.corner_radius,
            paint: color.into(),
        }
    }

    /// Sets the rectangle's corner radius.
    pub fn corner_radius(self, radius: f64) -> Rectangle {
        Rectangle {
            corner_radius: radius,
            paint: self.paint,
        }
    }
}

impl DynView for Rectangle {
    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        let rect = self.geom(path, ctx);
        let mut scene = Scene::new();
        scene.fill(
            peniko::Fill::EvenOdd,
            Affine::IDENTITY,
            &self.paint,
            None,
            &RoundedRect::from_rect(rect, self.corner_radius),
        );
        scene
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
        let rect = self.geom(path, ctx);

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

/// Renders a rectangle which expands to fill available space.
pub fn rectangle() -> Rectangle {
    Rectangle {
        corner_radius: 0.0,
        paint: AZURE_HIGHLIGHT.into(),
    }
}
