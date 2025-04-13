use vello::kurbo::Affine;

use crate::*;
use std::any::Any;
use std::hash::Hash;

#[derive(Clone, Copy)]
pub enum ListOrientation {
    Horizontal,
    Vertical,
    Z,
}

#[derive(Clone)]
pub struct List<ID, F> {
    orientation: ListOrientation,
    ids: Vec<ID>,
    func: F,
}

impl<ID, V, F> DynView for List<ID, F>
where
    ID: Hash + Clone + 'static,
    V: View,
    F: Fn(&ID) -> V + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        ctx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        for child in self.ids.iter().rev() {
            path.push(hh(child));
            let offset = ctx.get_layout(path).offset;
            ((self.func)(child)).process(&event.offset(-offset), path, ctx, actions);
            path.pop();
        }
    }

    fn draw(&self, path: &mut IdPath, ctx: &mut Context) -> Scene {
        let mut scene = Scene::new();
        for child in &self.ids {
            path.push(hh(child));

            let offset = ctx.get_layout(path).offset;

            let child_scene = ((self.func)(child)).draw(path, ctx);
            scene.append(&child_scene, Some(Affine::translate(offset)));

            path.pop();
        }
        scene
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> Size {
        match self.orientation {
            ListOrientation::Horizontal => {
                let n = self.ids.len() as f64;
                let proposed_child_size = Size::new(args.size.width / n, args.size.height);

                let mut sizes = Vec::<Size>::new();
                sizes.reserve(self.ids.len());

                let mut width_sum = 0.0;
                for child in &self.ids {
                    path.push(hh(child));
                    let child_size =
                        ((self.func)(child)).layout(path, &mut args.with_size(proposed_child_size));
                    sizes.push(child_size);
                    path.pop();

                    width_sum += child_size.width;
                }

                let mut max_height = 0.0;
                for size in &sizes {
                    max_height = size.height.max(max_height)
                }

                let mut x = 0.0;
                for c in 0..self.ids.len() {
                    path.push(hh(&self.ids[c]));
                    let child_size = sizes[c];

                    let child_offset = align_v(
                        Rect::from_origin_size(Point::ZERO, child_size),
                        Rect::from_origin_size((x, 0.0), (child_size.width, max_height)),
                        VAlignment::Middle,
                    );

                    args.ctx.set_layout_offset(path, child_offset);

                    path.pop();

                    x += child_size.width;
                }

                Size::new(width_sum, max_height)
            }
            ListOrientation::Vertical => {
                let n = self.ids.len() as f64;
                let proposed_child_size = Size::new(args.size.width, args.size.height / n);

                let mut sizes = Vec::<Size>::new();
                sizes.reserve(self.ids.len());

                let mut height_sum = 0.0;
                for child in &self.ids {
                    path.push(hh(child));
                    let child_size =
                        ((self.func)(child)).layout(path, &mut args.with_size(proposed_child_size));
                    sizes.push(child_size);
                    path.pop();

                    height_sum += child_size.height;
                }

                let mut max_width = 0.0;
                for size in &sizes {
                    max_width = size.width.max(max_width)
                }

                let mut y = height_sum;
                for c in 0..self.ids.len() {
                    path.push(hh(&self.ids[c]));
                    let child_size = sizes[c];

                    let child_offset = align_h(
                        Rect::from_origin_size(Point::ZERO, child_size),
                        Rect::from_origin_size(
                            (0.0, y - child_size.height),
                            (max_width, child_size.height),
                        ),
                        HAlignment::Center,
                    );

                    args.ctx.set_layout_offset(path, child_offset);
                    path.pop();

                    y -= child_size.height;
                }

                Size::new(max_width, height_sum)
            }
            ListOrientation::Z => {
                for child in &self.ids {
                    path.push(hh(child));
                    ((self.func)(child)).layout(path, args);
                    path.pop();
                }
                args.size
            }
        }
    }

    fn hittest(&self, path: &mut IdPath, pt: Point, ctx: &mut Context) -> Option<ViewId> {
        let mut hit = None;
        for child in &self.ids {
            path.push(hh(child));
            let offset = ctx.get_layout(path).offset;

            if let Some(h) = ((self.func)(child)).hittest(path, pt - offset, ctx) {
                hit = Some(h)
            }
            path.pop();
        }
        hit
    }

    fn commands(&self, path: &mut IdPath, ctx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        for child in &self.ids {
            path.push(hh(child));
            ((self.func)(child)).commands(path, ctx, cmds);
            path.pop();
        }
    }

    fn gc(&self, path: &mut IdPath, ctx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(ctx.view_id(path));
        for child in &self.ids {
            path.push(hh(child));
            map.push(ctx.view_id(path));
            ((self.func)(child)).gc(path, ctx, map);
            path.pop();
        }
    }
}

/// Displays a list of items all of which are represented by the same View. See `examples/list.rs`.
///
/// `ids` is a Vec of items that implement Hash.
///
/// `f` is a function called to generate a DynView for each item.
///
/// For example:
///
/// ```no_run
/// # use rui::*;
/// rui(list(vec![1, 2, 3], |i| {
///     hstack((
///         circle(),
///         text(&format!("{:?}", i))
///     ))
/// }));
/// ```
pub fn list<ID: Hash + Clone, V: View, F: Fn(&ID) -> V + Clone + 'static>(
    ids: Vec<ID>,
    f: F,
) -> List<ID, F> {
    List {
        orientation: ListOrientation::Vertical,
        ids,
        func: f,
    }
}

pub fn hlist<ID: Hash + Clone, V: View, F: Fn(&ID) -> V + Clone + 'static>(
    ids: Vec<ID>,
    f: F,
) -> List<ID, F> {
    List {
        orientation: ListOrientation::Horizontal,
        ids,
        func: f,
    }
}

pub fn zlist<ID: Hash + Clone, V: View, F: Fn(&ID) -> V + Clone + 'static>(
    ids: Vec<ID>,
    f: F,
) -> List<ID, F> {
    List {
        orientation: ListOrientation::Z,
        ids,
        func: f,
    }
}
