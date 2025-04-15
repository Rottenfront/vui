use crate::*;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

use std::time::Instant;
use winit::event::{ElementState, MouseButton, TouchPhase, WindowEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::{application::ApplicationHandler, event_loop::EventLoopProxy};

use vello::peniko::color::palette;
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, Renderer, RendererOptions};

use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowAttributes};

use vello::wgpu;

type WorkQueue = VecDeque<Box<dyn FnOnce(&mut Context) + Send>>;

lazy_static::lazy_static! {
    /// Allows us to wake the event loop whenever we want.
    static ref GLOBAL_EVENT_LOOP_PROXY: Mutex<Option<EventLoopProxy<()>>> = Mutex::new(None);

    static ref GLOBAL_WORK_QUEUE: Mutex<WorkQueue> = Mutex::new(WorkQueue::new());
}

fn default_threads() -> usize {
    /// Default number of threads to use for initializing shaders.
    ///
    /// This is 1 on macOS as it can be slower to use multiple threads for shader
    /// compilation on this platform, and 0 on other platforms, which allows the
    /// number of threads to be determined automatically.
    #[cfg(target_os = "macos")]
    return 1;
    #[cfg(not(target_os = "macos"))]
    return 0;
}

struct RenderState<'s> {
    // SAFETY: We MUST drop the surface before the `window`, so the fields
    // must be in this order
    surface: RenderSurface<'s>,
    window: Arc<Window>,
}

// TODO: Make this set configurable through the command line
// Alternatively, load anti-aliasing shaders on demand/asynchronously
const AA_CONFIGS: [AaConfig; 3] = [AaConfig::Area, AaConfig::Msaa8, AaConfig::Msaa16];

struct VelloApp<'s, T>
where
    T: View,
{
    context: RenderContext,
    renderers: Vec<Option<Renderer>>,
    state: Option<RenderState<'s>>,
    // Whilst suspended, we drop `render_state`, but need to keep the same window.
    // If render_state exists, we must store the window in it, to maintain drop order
    cached_window: Option<Arc<Window>>,

    use_cpu: bool,
    num_init_threads: usize,

    scene: Scene,

    stats_shown: bool,

    base_color: Option<Color>,

    complexity_shown: bool,
    vsync_on: bool,

    // We allow cycling through AA configs in either direction, so use a signed index
    aa_config_ix: i32,

    modifiers: ModifiersState,

    title: Arc<str>,
    mouse_position: Point,
    cx: Context,
    view: T,
}

fn process_event(cx: &mut Context, view: &impl View, event: &Event, window: Arc<Window>) {
    cx.process(view, event);

    if cx.grab_cursor && !cx.prev_grab_cursor {
        log::debug!("grabbing cursor");
        window
            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            .or_else(|_e| window.set_cursor_grab(winit::window::CursorGrabMode::Confined))
            .unwrap();
        window.set_cursor_visible(false);
    }

    if !cx.grab_cursor && cx.prev_grab_cursor {
        log::debug!("releasing cursor");
        window
            .set_cursor_grab(winit::window::CursorGrabMode::None)
            .unwrap();
        window.set_cursor_visible(true);
    }

    cx.prev_grab_cursor = cx.grab_cursor;
}

impl<T: View> ApplicationHandler<()> for VelloApp<'_, T> {
    #[cfg(target_arch = "wasm32")]
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    #[cfg(not(target_arch = "wasm32"))]
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let None = self.state else {
            return;
        };
        let window = self
            .cached_window
            .take()
            .unwrap_or_else(|| Arc::new(event_loop.create_window(window_attributes()).unwrap()));
        let size = window.inner_size();
        let present_mode = if self.vsync_on {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        let surface_future =
            self.context
                .create_surface(window.clone(), size.width, size.height, present_mode);
        // We need to block here, in case a Suspended event appeared
        let surface = pollster::block_on(surface_future).expect("Error creating surface");
        self.state = {
            let render_state = RenderState { window, surface };
            self.renderers
                .resize_with(self.context.devices.len(), || None);
            let id = render_state.surface.dev_id;
            self.renderers[id].get_or_insert_with(|| {
                let start = Instant::now();
                let renderer = Renderer::new(
                    &self.context.devices[id].device,
                    RendererOptions {
                        use_cpu: self.use_cpu,
                        antialiasing_support: AA_CONFIGS.iter().copied().collect(),
                        num_init_threads: NonZeroUsize::new(self.num_init_threads),
                    },
                )
                .map_err(|e| {
                    // Pretty-print any renderer creation error using Display formatting before unwrapping.
                    anyhow::format_err!("{e}")
                })
                .expect("Failed to create renderer");
                log::info!("Creating renderer {id} took {:?}", start.elapsed());
                renderer
            });
            Some(render_state)
        };
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(render_state) = &mut self.state else {
            return;
        };
        if render_state.window.id() != window_id {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::ModifiersChanged(m) => {
                self.modifiers = m.state();
                self.cx.key_mods = KeyboardModifiers {
                    shift: !(m.state() & ModifiersState::SHIFT).is_empty(),
                    control: !(m.state() & ModifiersState::CONTROL).is_empty(),
                    alt: !(m.state() & ModifiersState::ALT).is_empty(),
                    command: !(m.state() & ModifiersState::SUPER).is_empty(),
                };
            }
            WindowEvent::KeyboardInput { event, .. } => {
                {
                    let key = match event.logical_key.clone() {
                        Key::Named(NamedKey::Enter) => Some(event::Key::Enter),
                        Key::Named(NamedKey::Tab) => Some(event::Key::Tab),
                        Key::Named(NamedKey::Space) => Some(event::Key::Space),
                        Key::Named(NamedKey::ArrowDown) => Some(event::Key::ArrowDown),
                        Key::Named(NamedKey::ArrowLeft) => Some(event::Key::ArrowLeft),
                        Key::Named(NamedKey::ArrowRight) => Some(event::Key::ArrowRight),
                        Key::Named(NamedKey::ArrowUp) => Some(event::Key::ArrowUp),
                        Key::Named(NamedKey::End) => Some(event::Key::End),
                        Key::Named(NamedKey::Home) => Some(event::Key::Home),
                        Key::Named(NamedKey::PageDown) => Some(event::Key::PageDown),
                        Key::Named(NamedKey::PageUp) => Some(event::Key::PageUp),
                        Key::Named(NamedKey::Backspace) => Some(event::Key::Backspace),
                        Key::Named(NamedKey::Delete) => Some(event::Key::Delete),
                        Key::Named(NamedKey::Escape) => Some(event::Key::Escape),
                        Key::Named(NamedKey::F1) => Some(event::Key::F1),
                        Key::Named(NamedKey::F2) => Some(event::Key::F2),
                        Key::Named(NamedKey::F3) => Some(event::Key::F3),
                        Key::Named(NamedKey::F4) => Some(event::Key::F4),
                        Key::Named(NamedKey::F5) => Some(event::Key::F5),
                        Key::Named(NamedKey::F6) => Some(event::Key::F6),
                        Key::Named(NamedKey::F7) => Some(event::Key::F7),
                        Key::Named(NamedKey::F8) => Some(event::Key::F8),
                        Key::Named(NamedKey::F9) => Some(event::Key::F9),
                        Key::Named(NamedKey::F10) => Some(event::Key::F10),
                        Key::Named(NamedKey::F11) => Some(event::Key::F11),
                        Key::Named(NamedKey::F12) => Some(event::Key::F12),
                        Key::Character(str) => {
                            if let Some(c) = str.chars().next() {
                                Some(event::Key::Character(c))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    if let (Some(key), ElementState::Pressed) = (key, event.state) {
                        self.cx.process(&self.view, &Event::Key(key))
                    }

                    if let (Some(key), ElementState::Released) = (key, event.state) {
                        self.cx.process(&self.view, &Event::KeyReleased(key))
                    }
                }
                if event.state == ElementState::Pressed {
                    match event.logical_key.as_ref() {
                        Key::Character(char) => {
                            // TODO: Have a more principled way of handling modifiers on keypress
                            // see e.g. https://xi.zulipchat.com/#narrow/channel/351333-glazier/topic/Keyboard.20shortcuts/with/403538769
                            let char = char.to_lowercase();
                            match char.as_str() {
                                "s" => {
                                    self.stats_shown = !self.stats_shown;
                                }
                                "d" => {
                                    self.complexity_shown = !self.complexity_shown;
                                }
                                "m" => {
                                    self.aa_config_ix = if self.modifiers.shift_key() {
                                        self.aa_config_ix.saturating_sub(1)
                                    } else {
                                        self.aa_config_ix.saturating_add(1)
                                    };
                                }
                                "v" => {
                                    self.vsync_on = !self.vsync_on;
                                    self.context.set_present_mode(
                                        &mut render_state.surface,
                                        if self.vsync_on {
                                            wgpu::PresentMode::AutoVsync
                                        } else {
                                            wgpu::PresentMode::AutoNoVsync
                                        },
                                    );
                                }
                                _ => {}
                            }
                        }
                        Key::Named(NamedKey::Escape) => event_loop.exit(),
                        _ => {}
                    }
                }
            }
            WindowEvent::Touch(winit::event::Touch {
                phase, location, ..
            }) => {
                let position = (location.x, location.y).into();

                let delta = position - self.cx.previous_position[0];

                // TODO: Multi-Touch management
                let event = match phase {
                    TouchPhase::Started => Some(Event::TouchBegin { id: 0, position }),
                    TouchPhase::Moved => Some(Event::TouchMove {
                        id: 0,
                        position,
                        delta,
                    }),
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        Some(Event::TouchEnd { id: 0, position })
                    }
                };

                let window = self.cached_window.as_ref().unwrap().clone();

                if let Some(event) = event {
                    process_event(&mut self.cx, &self.view, &event, window);
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(RenderState { surface, window }) = &mut self.state {
                    self.context
                        .resize_surface(surface, size.width, size.height);
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                println!("{:?}", event);
                match state {
                    ElementState::Pressed => {
                        self.cx.mouse_button = match button {
                            MouseButton::Left => Some(event::MouseButton::Left),
                            MouseButton::Right => Some(event::MouseButton::Right),
                            MouseButton::Middle => Some(event::MouseButton::Center),
                            _ => None,
                        };

                        match button {
                            MouseButton::Left => self.cx.mouse_buttons.left = true,
                            MouseButton::Right => self.cx.mouse_buttons.right = true,
                            MouseButton::Middle => self.cx.mouse_buttons.middle = true,
                            _ => (),
                        };

                        self.cx.set_dirty();

                        if let Some(window) = &self.cached_window {
                            let event = Event::TouchBegin {
                                id: 0,
                                position: self.mouse_position,
                            };
                            process_event(&mut self.cx, &self.view, &event, window.clone());
                        }
                    }
                    ElementState::Released => {
                        self.cx.mouse_button = None;

                        match button {
                            MouseButton::Left => self.cx.mouse_buttons.left = false,
                            MouseButton::Right => self.cx.mouse_buttons.right = false,
                            MouseButton::Middle => self.cx.mouse_buttons.middle = false,
                            _ => (),
                        };

                        self.cx.set_dirty();

                        if let Some(window) = &self.cached_window {
                            let event = Event::TouchEnd {
                                id: 0,
                                position: self.mouse_position,
                            };
                            process_event(&mut self.cx, &self.view, &event, window.clone())
                        }
                    }
                };
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = (position.x, position.y).into();
            }
            WindowEvent::RedrawRequested => {
                render_state.window.request_redraw();

                let Some(RenderState { surface, .. }) = &self.state else {
                    return;
                };
                let width = surface.config.width;
                let height = surface.config.height;
                let device_handle = &self.context.devices[surface.dev_id];
                self.scene = self
                    .cx
                    .render(&self.view, (width as _, height as _).into(), 1.0);
                self.aa_config_ix = self.aa_config_ix.rem_euclid(AA_CONFIGS.len() as i32);

                // If the user specifies a base color in the CLI we use that. Otherwise we use any
                // color specified by the scene. The default is black.
                let base_color = self.base_color.unwrap_or(palette::css::BLACK);
                let antialiasing_method = AA_CONFIGS[self.aa_config_ix as usize];
                let render_params = vello::RenderParams {
                    base_color,
                    width,
                    height,
                    antialiasing_method,
                };

                self.renderers[surface.dev_id]
                    .as_mut()
                    .unwrap()
                    .render_to_texture(
                        &device_handle.device,
                        &device_handle.queue,
                        &self.scene,
                        &surface.target_view,
                        &render_params,
                    )
                    .expect("failed to render to texture");
                let surface_texture = surface
                    .surface
                    .get_current_texture()
                    .expect("failed to get surface texture");
                // Perform the copy
                // (TODO: Does it improve throughput to acquire the surface after the previous texture render has happened?)
                let mut encoder =
                    device_handle
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Surface Blit"),
                        });
                surface.blitter.copy(
                    &device_handle.device,
                    &mut encoder,
                    &surface.target_view,
                    &surface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                );
                device_handle.queue.submit([encoder.finish()]);
                surface_texture.present();
                {
                    device_handle.device.poll(wgpu::Maintain::Poll);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.cached_window {
            let width = window.inner_size().width;
            let height = window.inner_size().height;
            if self.cx.update(&self.view, (width as _, height as _).into()) {
                window.request_redraw();
            }

            if self.cx.window_title != self.title {
                self.title = self.cx.window_title.clone();
                window.set_title(&self.cx.window_title);
            }
        }
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        log::info!("Suspending");
        #[cfg(not(target_arch = "wasm32"))]
        // When we suspend, we need to remove the `wgpu` Surface
        if let Some(render_state) = self.state.take() {
            self.cached_window = Some(render_state.window);
        }
    }
}

fn run(event_loop: EventLoop<()>, render_cx: RenderContext, view: impl View) {
    let (render_state, renderers) = (None::<RenderState<'_>>, vec![]);

    let window_title = String::from("vui");
    let mut app = VelloApp {
        context: render_cx,
        renderers,
        state: render_state,
        cached_window: None,
        use_cpu: false,
        num_init_threads: default_threads(),
        scene: Scene::new(),
        stats_shown: true,
        base_color: None,
        complexity_shown: false,
        vsync_on: true,

        aa_config_ix: 0,
        view,
        title: window_title.into(),
        mouse_position: Point::ZERO,
        cx: Context::new(),

        modifiers: ModifiersState::default(),
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        *GLOBAL_EVENT_LOOP_PROXY.lock().unwrap() = Some(event_loop.create_proxy());
    }

    let mut commands: Vec<CommandInfo> = Vec::new();
    let mut command_map = HashMap::new();
    app.cx.commands(&app.view, &mut commands);

    {
        // So we can infer a type for CommandMap when winit is enabled.
        command_map.insert("", "");
    }

    if let Err(e) = event_loop.run_app(&mut app) {
        log::error!("Error exiting event loop: {:?}", e);
    };
}

fn window_attributes() -> WindowAttributes {
    Window::default_attributes()
        .with_inner_size(LogicalSize::new(1044, 800))
        .with_resizable(true)
        .with_title("Vello demo")
}

pub fn vui(view: impl View) -> anyhow::Result<()> {
    // TODO: initializing both env_logger and console_logger fails on wasm.
    // Figure out a more principled approach.
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::builder()
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .filter_level(log::LevelFilter::Warn)
        .init();
    let event_loop = EventLoop::<()>::with_user_event().build()?;
    let render_cx = RenderContext::new();

    run(event_loop, render_cx, view);

    Ok(())
}
