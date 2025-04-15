use crate::*;

pub const BUTTON_CORNER_RADIUS: f64 = 5.0;

#[derive(Default)]
struct ButtonState {
    hovered: bool,
    down: bool,
}

/// Calls a function when the button is tapped.
pub fn button<A: 'static, F: Fn(&mut Context) -> A + 'static + Clone>(
    view: impl View,
    f: F,
) -> impl View {
    state(
        || ButtonState::default(),
        move |s, ctx| {
            let f = f.clone();
            view.clone()
                .padding(Auto)
                .background(
                    rectangle()
                        .color(if ctx[s].down {
                            BUTTON_DOWN_COLOR
                        } else if ctx[s].hovered {
                            BUTTON_HOVER_COLOR
                        } else {
                            BUTTON_BACKGROUND_COLOR
                        })
                        .corner_radius(BUTTON_CORNER_RADIUS),
                )
                .touch(move |ctx, info| match info.state {
                    TouchState::Begin => {
                        ctx[s].down = true;
                    }
                    TouchState::End => {
                        ctx[s].down = false;
                        if ctx[s].hovered {
                            f(ctx);
                        }
                    }
                })
                .hover(move |ctx, inside| {
                    ctx[s].hovered = inside;
                })
        },
    )
}

/// Version of button which emits an action directly instead of taking a callback.
pub fn button_a<A: Clone + 'static>(view: impl View + Clone, action: A) -> impl View {
    button(view, move |_| action.clone())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_button() {
        let mut ctx = Context::new();

        let ui = state(
            || false,
            |pushed, _| {
                button("button", move |ctx| {
                    *pushed.get_mut(ctx) = true;
                })
            },
        );
        let size = (100.0, 100.0).into();

        let mut path = vec![0];
        let button_size = ui.layout(
            &mut path,
            &mut LayoutArgs {
                size,
                ctx: &mut ctx,
            },
        );
        assert!(path.len() == 1);

        assert_eq!(button_size, size);
        let s = StateHandle::<bool>::new(ctx.view_id(&path));
        assert!(!*s.get(&ctx));

        let events = [
            Event::TouchBegin {
                id: 0,
                position: (50.0, 50.0).into(),
            },
            Event::TouchEnd {
                id: 0,
                position: (50.0, 50.0).into(),
            },
        ];

        let mut actions = vec![];
        for event in &events {
            ui.process(event, &mut path, &mut ctx, &mut actions);
        }

        let vid = ctx.view_id(&path);
        assert!(ctx.state_map.contains_key(&vid));

        // State should have changed.
        assert!(*s.get(&ctx));
    }
}
