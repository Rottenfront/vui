use vello::kurbo::Affine;

use crate::views::stack_layout::*;
use crate::*;
use std::any::Any;

#[derive(Clone)]
pub enum StackOrientation {
    /// Views are stacked horizontally (right to left).
    Horizontal,

    /// Views are stacked vertically (top to bottom).
    Vertical,

    /// Views are stacked back to front.
    Z,
}

#[derive(Clone)]
pub struct Stack<VT, D> {
    children: VT,
    phantom_direction: std::marker::PhantomData<D>,
}

pub trait StackDirection: Clone {
    const ORIENTATION: StackOrientation;
}

#[derive(Clone)]
pub struct HorizontalDirection {}

impl StackDirection for HorizontalDirection {
    const ORIENTATION: StackOrientation = StackOrientation::Horizontal;
}

#[derive(Clone)]
pub struct VerticalDirection {}

impl StackDirection for VerticalDirection {
    const ORIENTATION: StackOrientation = StackOrientation::Vertical;
}

#[derive(Clone)]
pub struct ZDirection {}

impl StackDirection for ZDirection {
    const ORIENTATION: StackOrientation = StackOrientation::Z;
}

impl<VT: ViewTuple + 'static, D: StackDirection + 'static> DynView for Stack<VT, D> {
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let mut c = self.children.len() as i64 - 1;
        self.children.foreach_view_rev(&mut |child| {
            path.push(c as u64);
            let offset = ctx.get_layout(path).offset;
            (*child).process(&event.offset(-offset), path, ctx, actions);
            path.pop();
            c -= 1;
        })
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        let mut scene = Scene::new();
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            path.push(c);
            let layout_box = ctx.get_layout(path);

            let child_scene = (*child).draw(path, ctx);
            c += 1;

            path.pop();
            scene.append(&child_scene, Some(Affine::translate(layout_box.offset)));
        });
        scene
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        let n = self.children.len() as f64;

        match D::ORIENTATION {
            StackOrientation::Horizontal => {
                let proposed_child_size = Size::new(args.size.width / n, args.size.height);

                let mut child_sizes = [None; VIEW_TUPLE_MAX_ELEMENTS];
                self.layout_fixed_children(path, proposed_child_size, args, &mut child_sizes);

                let child_sizes_1d = child_sizes.map(|x| {
                    if let Some(s) = x {
                        StackItem::Fixed(s.width)
                    } else {
                        StackItem::Flexible
                    }
                });
                let mut intervals = [(0.0, 0.0); VIEW_TUPLE_MAX_ELEMENTS];
                let n = self.children.len();
                let mut flex_length = 0.0;
                let length = stack_layout(
                    args.size.width,
                    &child_sizes_1d[0..n],
                    &mut intervals[0..n],
                    &mut flex_length,
                );

                self.layout_flex_children(
                    path,
                    (flex_length, args.size.height).into(),
                    args,
                    &mut child_sizes,
                );

                let mut max_height = 0.0;
                for size in &child_sizes[0..self.children.len()] {
                    max_height = size.unwrap().height.max(max_height)
                }

                for c in 0..(self.children.len() as u64) {
                    let ab = intervals[c as usize];

                    let child_offset = align_v(
                        Rect::from_origin_size(Point::ZERO, child_sizes[c as usize].unwrap()),
                        Rect::from_origin_size((ab.0, 0.0), (ab.1 - ab.0, max_height)),
                        VAlignment::Middle,
                    );

                    path.push(c);
                    args.ctx.set_layout_offset(path, child_offset);
                    path.pop();
                }

                (length, max_height).into()
            }
            StackOrientation::Vertical => {
                let proposed_child_size = Size::new(args.size.width, args.size.height / n);
                let mut child_sizes = [None; VIEW_TUPLE_MAX_ELEMENTS];
                self.layout_fixed_children(path, proposed_child_size, args, &mut child_sizes);

                let child_sizes_1d = child_sizes.map(|x| {
                    if let Some(s) = x {
                        StackItem::Fixed(s.height)
                    } else {
                        StackItem::Flexible
                    }
                });
                let mut intervals = [(0.0, 0.0); VIEW_TUPLE_MAX_ELEMENTS];
                let n = self.children.len();
                let mut flex_length = 0.0;
                let length = stack_layout(
                    args.size.height,
                    &child_sizes_1d[0..n],
                    &mut intervals[0..n],
                    &mut flex_length,
                );

                self.layout_flex_children(
                    path,
                    (args.size.width, flex_length).into(),
                    args,
                    &mut child_sizes,
                );

                let mut max_width = 0.0;
                for size in &child_sizes[0..self.children.len()] {
                    max_width = size.unwrap().width.max(max_width)
                }

                for c in 0..(self.children.len() as u64) {
                    let ab = intervals[c as usize];

                    let h = ab.1 - ab.0;
                    let child_offset = align_h(
                        Rect::from_origin_size(Point::ZERO, child_sizes[c as usize].unwrap()),
                        Rect::from_origin_size((0.0, length - ab.0 - h), (max_width, h)),
                        HAlignment::Center,
                    );

                    path.push(c);
                    args.ctx.set_layout_offset(path, child_offset);
                    path.pop();
                }

                (max_width, length).into()
            }
            StackOrientation::Z => {
                let mut c = 0;
                self.children.foreach_view(&mut |child| {
                    path.push(c);
                    child.layout(path, args);
                    path.pop();
                    c += 1;
                });
                args.size
            }
        }
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        let mut c = 0;
        let mut hit = None;
        self.children.foreach_view(&mut |child| {
            path.push(c);
            let offset = ctx.get_layout(path).offset;

            if let Some(h) = child.hittest(path, pt - offset, ctx) {
                hit = Some(h)
            }

            path.pop();

            c += 1;
        });
        hit
    }

    fn commands(&self, path: &mut IdPath, ctx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            path.push(c);
            child.commands(path, ctx, cmds);
            path.pop();
            c += 1;
        });
    }

    fn gc(&self, path: &mut IdPath, ctx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(ctx.view_id(path));
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            path.push(c);
            map.push(ctx.view_id(path));
            child.gc(path, ctx, map);
            path.pop();
            c += 1;
        });
    }
}

impl<VT: ViewTuple, D: StackDirection> Stack<VT, D> {
    pub fn new(children: VT) -> Self {
        Self {
            children,
            phantom_direction: std::marker::PhantomData::default(),
        }
    }

    pub fn layout_fixed_children(
        &self,
        path: &mut IdPath,
        proposed_child_size: Size,
        args: &mut LayoutArgs,
        child_sizes: &mut [Option<Size>],
    ) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            path.push(c);
            if !child.is_flexible() {
                child_sizes[c as usize] =
                    Some(child.layout(path, &mut args.with_size(proposed_child_size)))
            }
            path.pop();
            c += 1;
        });
    }

    pub fn layout_flex_children(
        &self,
        path: &mut IdPath,
        flex_size: Size,
        args: &mut LayoutArgs,
        child_sizes: &mut [Option<Size>],
    ) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            path.push(c);
            if child.is_flexible() {
                child_sizes[c as usize] = Some(child.layout(path, &mut args.with_size(flex_size)));
            }
            path.pop();
            c += 1;
        });
    }
}

/// Horizontal stack of up to 128 Views in a tuple. Each item can be a different view type.
pub fn hstack<VT: ViewTuple + 'static>(children: VT) -> Stack<VT, HorizontalDirection> {
    Stack::<VT, HorizontalDirection>::new(children)
}

/// Vertical stack of up to 128 Views in a tuple. Each item can be a different view type.
pub fn vstack<VT: ViewTuple + 'static>(children: VT) -> Stack<VT, VerticalDirection> {
    Stack::<VT, VerticalDirection>::new(children)
}

/// Stack of up to 128 overlaid Views in a tuple. Each item can be a different view type.
pub fn zstack<VT: ViewTuple + 'static>(children: VT) -> Stack<VT, ZDirection> {
    Stack::<VT, ZDirection>::new(children)
}
