use crate::*;
use vello::kurbo::Affine;

const SLIDER_WIDTH: f64 = 4.0;
const SLIDER_THUMB_RADIUS: f64 = 10.0;

#[derive(Clone, Copy)]
pub struct SliderOptions {
    thumb: Color,
}

impl Default for SliderOptions {
    fn default() -> Self {
        Self {
            thumb: AZURE_HIGHLIGHT,
        }
    }
}

pub trait SliderMods: View + Sized {
    fn thumb_color(self, color: Color) -> Self;
}

/// Horizontal slider built from other Views.
pub fn hslider(value: impl Binding<f64>) -> impl SliderMods {
    modview(move |opts: SliderOptions, _| {
        state(
            || 0.0,
            move |width, ctx| {
                let w = ctx[width];
                canvas(move |ctx, size| {
                    let mut scene = Scene::new();
                    let c = size.center();

                    let w = ctx[width];
                    let v = value.get(ctx);
                    let r = SLIDER_THUMB_RADIUS;
                    let start_x = r;
                    let end_x = w - r;
                    let x = (1.0 - v) * start_x + v * (end_x);

                    scene.fill(
                        peniko::Fill::NonZero,
                        Affine::IDENTITY,
                        &BUTTON_BACKGROUND_COLOR,
                        None,
                        &Rect::new(
                            start_x,
                            c.y - SLIDER_WIDTH / 2.0,
                            size.width() - 2.0 * r,
                            SLIDER_WIDTH,
                        ),
                    );
                    scene.fill(
                        peniko::Fill::NonZero,
                        Affine::IDENTITY,
                        &AZURE_HIGHLIGHT_BACKGROUND,
                        None,
                        &Rect::new(start_x, c.y - SLIDER_WIDTH / 2.0, x, SLIDER_WIDTH),
                    );
                    scene.fill(
                        peniko::Fill::NonZero,
                        Affine::IDENTITY,
                        &opts.thumb,
                        None,
                        &kurbo::Circle::new((x, c.y), r),
                    );
                    scene
                })
                .geom(move |ctx, size| {
                    if size.width != ctx[width] {
                        ctx[width] = size.width;
                    }
                })
                .drag_s(value, move |v, delta, _, _| {
                    *v = (*v + delta.x / w).clamp(0.0, 1.0)
                })
            },
        )
    })
}

impl<F> SliderMods for ModView<SliderOptions, F>
where
    ModView<SliderOptions, F>: View,
{
    fn thumb_color(self, color: Color) -> Self {
        let mut opts = self.value;
        opts.thumb = color;
        ModView {
            func: self.func,
            value: opts,
        }
    }
}

/// Vertical slider built from other Views.
pub fn vslider(
    value: f64,
    set_value: impl Fn(&mut Context, f64) + 'static + Copy,
) -> impl SliderMods {
    modview(move |opts: SliderOptions, _| {
        state(
            || 0.0,
            move |height, _| {
                canvas(move |ctx, size| {
                    let mut scene = Scene::new();
                    let h = ctx[height];
                    let y = value * h;
                    let c = size.center();
                    scene.fill(
                        peniko::Fill::NonZero,
                        Affine::IDENTITY,
                        &BUTTON_BACKGROUND_COLOR,
                        None,
                        &Rect::new(c.x - SLIDER_WIDTH / 2.0, 0.0, SLIDER_WIDTH, size.height()),
                    );
                    scene.fill(
                        peniko::Fill::NonZero,
                        Affine::IDENTITY,
                        &opts.thumb,
                        None,
                        &kurbo::Circle::new((c.x, y), SLIDER_THUMB_RADIUS),
                    );
                    scene
                })
                .geom(move |ctx, size| {
                    if size.height != ctx[height] {
                        ctx[height] = size.height;
                    }
                })
                .drag(move |ctx, delta, _, _| {
                    (set_value)(ctx, (value + delta.y / ctx[height]).clamp(0.0, 1.0));
                })
            },
        )
    })
}
