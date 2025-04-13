use crate::*;
use parley::FontContext;
use std::any::Any;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::ops;
use std::sync::Arc;
use vello::Scene;

#[derive(Clone, Eq, PartialEq)]
pub struct CommandInfo {
    pub path: Arc<str>,
    pub key: Option<HotKey>,
}

pub const DEBUG_LAYOUT: bool = false;

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub(crate) struct LayoutBox {
    pub rect: Rect,
    pub offset: Vec2,
}

pub(crate) struct StateHolder {
    pub state: Box<dyn Any>,
    pub dirty: bool,
}

pub(crate) type StateMap = HashMap<ViewId, StateHolder>;

pub(crate) type EnvMap = HashMap<TypeId, Box<dyn Any>>;

/// The Context stores all UI state. A user of the library
/// shouldn't have to interact with it directly.
pub struct Context {
    /// Layout information for all views.
    layout: HashMap<IdPath, LayoutBox>,

    /// Allocated ViewIds.
    view_ids: HashMap<IdPath, ViewId>,

    /// Next allocated id.
    next_id: ViewId,

    /// Which views each touch (or mouse pointer) is interacting with.
    pub(crate) touches: [ViewId; 16],

    /// Points at which touches (or click-drags) started.
    pub(crate) starts: [Point; 16],

    /// Previous touch/mouse positions.
    pub(crate) previous_position: [Point; 16],

    /// Current mouse button for event handling.
    pub(crate) mouse_button: Option<MouseButton>,

    /// Mouse button state.
    pub mouse_buttons: MouseButtons,

    /// Keyboard modifiers state.
    pub key_mods: KeyboardModifiers,

    /// The view that has the keyboard focus.
    pub(crate) focused_id: Option<ViewId>,

    /// The current title of the window
    pub window_title: Arc<str>,

    /// Are we fullscreen?
    pub fullscreen: bool,

    /// User state created by `state`.
    pub(crate) state_map: StateMap,

    /// Has the state changed?
    pub(crate) dirty: bool,

    /// Are we currently setting the dirty bit?
    pub(crate) enable_dirty: bool,

    /// Values indexed by type.
    pub(crate) env: EnvMap,

    /// State dependencies.
    pub(crate) deps: HashMap<ViewId, Vec<ViewId>>,

    /// A stack of ids for states to get parent dependencies.
    pub(crate) id_stack: Vec<ViewId>,

    /// Previous window size.
    window_size: Size,

    /// Offset for events at the root level.
    root_offset: Vec2,

    /// Lock the cursor in position. Useful for dragging knobs.
    pub(crate) grab_cursor: bool,

    pub font_ctx: FontContext,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            layout: HashMap::new(),
            view_ids: HashMap::new(),
            next_id: ViewId { id: 0 },
            touches: [ViewId::default(); 16],
            starts: [Point::ZERO; 16],
            previous_position: [Point::ZERO; 16],
            mouse_button: None,
            mouse_buttons: Default::default(),
            key_mods: Default::default(),
            focused_id: None,
            window_title: "rui".into(),
            fullscreen: false,
            state_map: HashMap::new(),
            dirty: false,
            enable_dirty: true,
            env: HashMap::new(),
            deps: HashMap::new(),
            id_stack: vec![],
            window_size: Size::default(),
            root_offset: Vec2::ZERO,
            grab_cursor: false,
            font_ctx: FontContext::new(),
        }
    }

    /// Call this after the event queue is cleared.
    pub fn update(&mut self, view: &impl View, window_size: Size) -> bool {
        // If the window size has changed, force a relayout.
        if window_size != self.window_size {
            self.deps.clear();
            self.window_size = window_size;
        }

        let mut path = vec![0];

        // Run any animations.
        let mut actions = vec![];
        view.process(&Event::Anim, &mut path, self, &mut actions);
        assert!(path.len() == 1);

        if self.dirty {
            // Clean up state and layout.
            let mut keep = vec![];
            view.gc(&mut path, self, &mut keep);
            assert!(path.len() == 1);
            let keep_set = HashSet::<ViewId>::from_iter(keep);
            self.state_map.retain(|k, _| keep_set.contains(k));

            let mut new_layout = self.layout.clone();
            new_layout.retain(|k, _| keep_set.contains(&self.view_id(k)));
            self.layout = new_layout;

            // XXX: we're doing layout both here and in rendering.
            view.layout(
                &mut path,
                &mut LayoutArgs {
                    size: window_size,
                    ctx: self,
                },
            );
            assert_eq!(path.len(), 1);

            self.clear_dirty();

            true
        } else {
            false
        }
    }

    /// Redraw the UI
    pub fn render(&mut self, view: &impl View, window_size: Size, _scale: f32) -> Scene {
        let mut path = vec![0];
        // Disable dirtying the state during layout and rendering
        // to avoid constantly re-rendering if some state is saved.
        self.enable_dirty = false;
        let _ = view.layout(
            &mut path,
            &mut LayoutArgs {
                size: window_size,
                ctx: self,
            },
        );
        assert!(path.len() == 1);

        // Center the root view in the window.
        self.root_offset = Vec2::ZERO;

        let scene = view.draw(&mut path, self);
        self.enable_dirty = true;

        scene
    }

    /// Process a UI event
    pub fn process(&mut self, view: &impl View, event: &Event) {
        let mut actions = vec![];
        let mut path = vec![0];
        view.process(
            &event.offset(-self.root_offset),
            &mut path,
            self,
            &mut actions,
        );

        for action in actions {
            if !action.is::<()>() {
                log::debug!("unhandled action: {:?}", action.type_id());
            }
        }
    }

    /// Get menu commands.
    pub fn commands(&mut self, view: &impl View, cmds: &mut Vec<CommandInfo>) {
        let mut path = vec![0];
        view.commands(&mut path, self, cmds);
    }

    pub(crate) fn view_id(&mut self, path: &IdPath) -> ViewId {
        match self.view_ids.get_mut(path) {
            Some(id) => *id,
            None => {
                let id = self.next_id;
                self.view_ids.insert(path.clone(), id);
                self.next_id.id += 1;
                id
            }
        }
    }

    pub(crate) fn get_layout(&self, path: &IdPath) -> LayoutBox {
        match self.layout.get(path) {
            Some(b) => *b,
            None => LayoutBox::default(),
        }
    }

    pub(crate) fn update_layout(&mut self, path: &IdPath, layout_box: LayoutBox) {
        match self.layout.get_mut(path) {
            Some(bref) => *bref = layout_box,
            None => {
                self.layout.insert(path.clone(), layout_box);
            }
        }
    }

    pub(crate) fn set_layout_offset(&mut self, path: &IdPath, offset: Vec2) {
        match self.layout.get_mut(path) {
            Some(boxref) => boxref.offset = offset,
            None => {
                self.layout.insert(
                    path.clone(),
                    LayoutBox {
                        rect: Rect::default(),
                        offset: offset,
                    },
                );
            }
        }
    }

    pub(crate) fn set_dirty(&mut self) {
        if self.enable_dirty {
            self.dirty = true
        }
    }

    pub(crate) fn clear_dirty(&mut self) {
        self.dirty = false;
        for holder in &mut self.state_map.values_mut() {
            holder.dirty = false;
        }
    }

    pub(crate) fn set_state<S: 'static>(&mut self, id: ViewId, value: S) {
        self.state_map.insert(
            id,
            StateHolder {
                state: Box::new(value),
                dirty: false,
            },
        );
    }

    pub(crate) fn is_dirty(&self, id: ViewId) -> bool {
        self.state_map[&id].dirty
    }

    pub(crate) fn init_state<S: 'static, D: Fn() -> S + 'static>(&mut self, id: ViewId, func: &D) {
        self.state_map.entry(id).or_insert_with(|| StateHolder {
            state: Box::new((func)()),
            dirty: false,
        });
    }

    pub(crate) fn init_env<S: Clone + 'static, D: Fn() -> S + 'static>(&mut self, func: &D) -> S {
        self.env
            .entry(TypeId::of::<S>())
            .or_insert_with(|| Box::new((func)()))
            .downcast_ref::<S>()
            .unwrap()
            .clone()
    }

    pub(crate) fn set_env<S: Clone + 'static>(&mut self, value: &S) -> Option<S> {
        let typeid = TypeId::of::<S>();
        let old_value = self
            .env
            .get(&typeid)
            .map(|b| b.downcast_ref::<S>().unwrap().clone());
        self.env.insert(typeid, Box::new(value.clone()));
        old_value
    }

    pub fn get<S>(&self, id: StateHandle<S>) -> &S
    where
        S: 'static,
    {
        self.state_map[&id.id].state.downcast_ref().unwrap()
    }

    pub fn get_mut<S>(&mut self, id: StateHandle<S>) -> &mut S
    where
        S: 'static,
    {
        self.set_dirty();

        let holder = self.state_map.get_mut(&id.id).unwrap();
        holder.dirty = true;
        holder.state.downcast_mut().unwrap()
    }
}

impl<S> ops::Index<StateHandle<S>> for Context
where
    S: 'static,
{
    type Output = S;

    fn index(&self, index: StateHandle<S>) -> &S {
        self.get(index)
    }
}

impl<S> ops::IndexMut<StateHandle<S>> for Context
where
    S: 'static,
{
    fn index_mut(&mut self, index: StateHandle<S>) -> &mut S {
        self.get_mut(index)
    }
}
