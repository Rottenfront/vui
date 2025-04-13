use parley::{
    AlignmentOptions, FontContext, GenericFamily, Layout, LayoutContext, PositionedLayoutItem,
    StyleProperty,
};
use vello::kurbo::Affine;

use crate::*;

pub trait TextModifiers: View + Sized {
    fn font_size(self, size: f32) -> Text;
    fn color(self, color: Color) -> Text;
    fn max_width(self, max_width: f32) -> Text;
}

/// Struct for `text`.
#[derive(Clone)]
pub struct Text {
    text: String,
    size: f32,
    color: Color,
    max_width: Option<f32>,
}

impl Text {
    pub const DEFAULT_SIZE: f32 = 14.0;
    pub fn color(self, color: Color) -> Text {
        Text {
            text: self.text,
            size: self.size,
            color,
            max_width: self.max_width,
        }
    }
}

impl DynView for Text {
    fn draw(&self, _path: &mut IdPath, ctx: &mut Context) -> Scene {
        draw_text(
            &self.text,
            self.size,
            Vec2::ZERO,
            self.max_width,
            TEXT_COLOR,
            &mut ctx.font_ctx,
        )
    }
    fn layout(&self, _path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        let width = match self.max_width {
            None => args.size.width as _,
            Some(max_width) => max_width.min(args.size.width as _),
        };
        get_text_bounds(
            &self.text,
            self.size,
            Some(width as _),
            &mut args.ctx.font_ctx,
        )
    }
    fn hittest(&self, _path: &mut IdPath, _pt: Point, _ctx: &mut Context) -> Option<ViewId> {
        None
    }
}

impl TextModifiers for Text {
    fn font_size(self, size: f32) -> Self {
        Self {
            text: self.text,
            color: self.color,
            size,
            max_width: self.max_width,
        }
    }
    fn color(self, color: Color) -> Text {
        Text {
            text: self.text,
            size: self.size,
            color,
            max_width: self.max_width,
        }
    }
    fn max_width(self, max_width: f32) -> Text {
        Text {
            text: self.text,
            size: self.size,
            color: self.color,
            max_width: Some(max_width),
        }
    }
}

/// Shows a string as a label (not editable).
pub fn text(name: &str) -> Text {
    Text {
        text: String::from(name),
        size: Text::DEFAULT_SIZE,
        color: TEXT_COLOR,
        max_width: None,
    }
}

macro_rules! impl_text {
    ( $ty:ident ) => {
        impl DynView for $ty {
            fn draw(&self, _path: &mut IdPath, ctx: &mut Context) -> Scene {
                let text = &format!("{}", self);
                draw_text(
                    text,
                    Text::DEFAULT_SIZE,
                    Vec2::ZERO,
                    None,
                    TEXT_COLOR,
                    &mut ctx.font_ctx,
                )
            }
            fn layout(&self, _path: &mut IdPath, args: &mut LayoutArgs) -> Size {
                let text = &format!("{}", self);
                let width = args.size.width;
                get_text_bounds(
                    text,
                    Text::DEFAULT_SIZE,
                    Some(width as _),
                    &mut args.ctx.font_ctx,
                )
            }
        }

        impl TextModifiers for $ty {
            fn font_size(self, size: f32) -> Text {
                Text {
                    text: format!("{}", self),
                    size,
                    color: TEXT_COLOR,
                    max_width: None,
                }
            }
            fn color(self, color: Color) -> Text {
                Text {
                    text: format!("{}", self),
                    size: Text::DEFAULT_SIZE,
                    color,
                    max_width: None,
                }
            }

            fn max_width(self, max_width: f32) -> Text {
                Text {
                    text: format!("{}", self),
                    size: Text::DEFAULT_SIZE,
                    color: TEXT_COLOR,
                    max_width: Some(max_width),
                }
            }
        }
    };
}

// XXX: this used to be generic for any Display but
//      that was causing trouble with adding Clone to view.
//      Perhaps a rust wizard can figure out why.
impl_text!(String);
impl_text!(u32);
impl_text!(i32);
impl_text!(u64);
impl_text!(i64);
impl_text!(f32);
impl_text!(f64);

// XXX: Can't do impl_text!(&'static str)
impl DynView for &'static str {
    fn draw(&self, _path: &mut IdPath, ctx: &mut Context) -> Scene {
        draw_text(
            self,
            Text::DEFAULT_SIZE,
            Vec2::ZERO,
            None,
            TEXT_COLOR,
            &mut ctx.font_ctx,
        )
    }
    fn layout(&self, _path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        let width = args.size.width;
        get_text_bounds(
            self,
            Text::DEFAULT_SIZE,
            Some(width as _),
            &mut args.ctx.font_ctx,
        )
    }
}

impl TextModifiers for &'static str {
    fn font_size(self, size: f32) -> Text {
        Text {
            text: format!("{}", self),
            size,
            color: TEXT_COLOR,
            max_width: None,
        }
    }
    fn color(self, color: Color) -> Text {
        Text {
            text: format!("{}", self),
            size: Text::DEFAULT_SIZE,
            color,
            max_width: None,
        }
    }
    fn max_width(self, max_width: f32) -> Text {
        Text {
            text: format!("{}", self),
            size: Text::DEFAULT_SIZE,
            color: TEXT_COLOR,
            max_width: Some(max_width),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ColorBrush {
    color: Color,
}

impl Default for ColorBrush {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
        }
    }
}

fn build_layout(
    text: &str,
    font_size: f32,
    max_width: Option<f32>,
    color: Option<Color>,
    font_ctx: &mut FontContext,
) -> Layout<ColorBrush> {
    let mut layout_ctx = LayoutContext::new();
    // todo: make everything scalable
    let mut builder = layout_ctx.ranged_builder(font_ctx, text, 1.0);
    // todo: add font system
    builder.push_default(GenericFamily::SystemUi);
    builder.push_default(StyleProperty::FontSize(font_size));
    builder.push_default(StyleProperty::LineHeight(1.2));
    if let Some(color) = color {
        builder.push_default(StyleProperty::Brush(ColorBrush { color }));
    }
    let mut layout: Layout<ColorBrush> = builder.build(text);
    layout.break_all_lines(max_width);
    layout.align(
        max_width,
        parley::Alignment::Start,
        AlignmentOptions::default(),
    );

    layout
}

fn get_text_bounds(
    text: &str,
    font_size: f32,
    max_width: Option<f32>,
    font_ctx: &mut FontContext,
) -> Size {
    let layout = build_layout(text, font_size, max_width, None, font_ctx);
    (layout.full_width() as f64, layout.height() as f64).into()
}

fn draw_text(
    text: &str,
    font_size: f32,
    offset: Vec2,
    max_width: Option<f32>,
    color: Color,
    font_ctx: &mut FontContext,
) -> Scene {
    let mut scene = Scene::new();
    let layout = build_layout(text, font_size, max_width, Some(color), font_ctx);
    let transform = Affine::translate(offset);

    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let style = glyph_run.style();
            let mut x = glyph_run.offset();
            let y = glyph_run.baseline();
            let run = glyph_run.run();
            let font = run.font();
            let font_size = run.font_size();
            let synthesis = run.synthesis();
            let glyph_xform = synthesis
                .skew()
                .map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0));
            scene
                .draw_glyphs(font)
                .brush(&style.brush.color)
                .hint(true)
                .transform(transform)
                .glyph_transform(glyph_xform)
                .font_size(font_size)
                .normalized_coords(run.normalized_coords())
                .draw(
                    peniko::Fill::NonZero,
                    glyph_run.glyphs().map(|glyph| {
                        let gx = x + glyph.x;
                        let gy = y - glyph.y;
                        x += glyph.advance;
                        vello::Glyph {
                            id: glyph.id as _,
                            x: gx,
                            y: gy,
                        }
                    }),
                );
        }
    }

    scene
}
