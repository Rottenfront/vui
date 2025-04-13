use vello::peniko::color::palette::css::CYAN;

use crate::*;

pub struct Toggle;

impl Toggle {
    pub fn new() -> ToggleConfig {
        ToggleConfig::default()
    }
}

#[derive(Clone)]
pub struct ToggleConfig {
    width: f64,
    height: f64,
    edge: f64,
    animation_speed: f64,
    background_on: Color,
    background_off: Color,
    knob_color: Color,
}

impl Default for ToggleConfig {
    fn default() -> Self {
        Self {
            width: 35.0,
            height: 20.0,
            edge: 1.0,
            animation_speed: 15.0,
            background_on: AZURE_HIGHLIGHT_BACKGROUND,
            background_off: CONTROL_BACKGROUND,
            knob_color: MEDIUM_GRAY,
        }
    }
}

impl ToggleConfig {
    pub fn width(mut self, width: f64) -> Self {
        self.width = width.max(self.height); // Ensure width is at least as large as height
        self
    }

    pub fn height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    pub fn edge(mut self, edge: f64) -> Self {
        self.edge = edge.max(0.0); // Prevent negative edge values
        self
    }

    pub fn animation_speed(mut self, speed: f64) -> Self {
        self.animation_speed = speed.max(0.0); // Prevent negative animation speed
        self
    }

    pub fn colors(mut self, background_on: Color, background_off: Color, knob: Color) -> Self {
        self.background_on = background_on;
        self.background_off = background_off;
        self.knob_color = knob;
        self
    }

    pub fn show(self, on: impl Binding<bool>) -> impl View {
        toggle_with_config(on, self)
    }
}

pub fn toggle_with_config(on: impl Binding<bool>, config: ToggleConfig) -> impl View {
    let double_edge = config.edge * 2.0;
    let knob_size = config.height - double_edge;
    let travel_distance = config.width - config.height;

    state(
        || 0.0f64,
        move |animation: StateHandle<f64>, ctx| {
            let is_on = *on.get(ctx);

            let animation_speed = config.animation_speed;

            zstack((
                // Background rectangle
                rectangle(if is_on {
                    config.background_on
                } else {
                    config.background_off
                })
                .corner_radius(config.height / 2.0) // Make corners perfectly round
                .sized((config.width, config.height))
                .tap(move |ctx| {
                    on.with_mut(ctx, |b| *b = !*b);
                }),
                // .hover_cursor(CursorStyle::Pointer), // Add pointer cursor on hover
                // Knob circle
                circle(CYAN)
                    .color(config.knob_color)
                    .sized((knob_size, knob_size))
                    .offset((ctx[animation] * travel_distance + config.edge, config.edge)), // .shadow(2.0, [0.0, 1.0], Color::BLACK.with_alpha(0.1)), // Add subtle shadow
            ))
            .anim(move |ctx, dt| {
                let target = if is_on { 1.0 } else { 0.0 };
                let delta = target - ctx[animation];

                ctx[animation] += delta * animation_speed * dt;

                // Snap to final position when very close
                if delta.abs() < 0.01 {
                    ctx[animation] = target;
                }
            })
        },
    )
}

/// A toggle switch with default configuration.
pub fn toggle(on: impl Binding<bool>) -> impl View {
    toggle_with_config(on, ToggleConfig::default())
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_toggle() {
        let mut ctx = Context::new();

        let ui = state(|| false, |s, _| toggle(s));
        let size = (40.0, 20.0).into();

        let mut path = vec![0];
        let knob_size = ui.layout(
            &mut path,
            &mut LayoutArgs {
                size,
                ctx: &mut ctx,
            },
        );

        assert_eq!(knob_size, size);
        let s = StateHandle::<bool>::new(ctx.view_id(&path));
        assert_eq!(*s.get(&ctx), false);

        let events = [
            Event::TouchBegin {
                id: 0,
                position: (10.0, 10.0).into(),
            },
            Event::TouchEnd {
                id: 0,
                position: (10.0, 10.0).into(),
            },
        ];

        let mut actions = vec![];
        for event in &events {
            ui.process(event, &mut path, &mut ctx, &mut actions);
        }

        // State should have changed.
        assert_eq!(*s.get(&ctx), true);
    }
}
