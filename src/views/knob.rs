use vello::kurbo::Affine;

use crate::*;

const THETA_MIN: f64 = 3.0 / 2.0 * std::f64::consts::PI;
const THETA_MAX: f64 = 7.0 / 2.0 * std::f64::consts::PI;

fn lerp(x: f64, a: f64, b: f64) -> f64 {
    (1.0 - x) * a + x * b
}

/// Knob for controlling a 0 to 1 floating point parameter.
pub fn knob(value: impl Binding<f64>) -> impl View {
    zstack((
        circle(CLEAR_COLOR)
            .drag_s(value, move |v, delta, _, _| {
                *v = (*v + (delta.x + delta.y) / 400.0).clamp(0.0, 1.0)
            })
            .grab_cursor(),
        canvas(move |ctx, size| {
            let mut scene = Scene::new();
            let c = size.center();
            let r = size.width().min(size.height()) / 2.0;

            scene.stroke(
                &kurbo::Stroke::new(2.0),
                Affine::IDENTITY,
                &CONTROL_BACKGROUND,
                None,
                &kurbo::Arc::new(c, (r, r), 0.0, 0.0, std::f64::consts::PI),
            );

            let a0 = lerp(*value.get(ctx), THETA_MAX, THETA_MIN);
            let a1 = THETA_MAX;

            let theta = -(a0 + a1) / 2.0 + std::f64::consts::PI;
            let ap = (a0 - a1).abs() / 2.0;

            scene.stroke(
                &kurbo::Stroke::new(2.0),
                Affine::IDENTITY,
                &AZURE_HIGHLIGHT,
                None,
                &kurbo::Arc::new(c, (r, r), theta, 0.0, ap),
            );
            scene
        }),
    ))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_knob() {
        let mut ctx = Context::new();

        let ui = state(|| 0.0, |s, _| knob(s));
        let size = (100.0, 100.0).into();

        let mut path = vec![0];
        let knob_size = ui.layout(
            &mut path,
            &mut LayoutArgs {
                size,
                ctx: &mut ctx,
            },
        );

        assert_eq!(knob_size, size);
        let s = StateHandle::<f64>::new(ctx.view_id(&path));
        assert_eq!(*s.get(&ctx), 0.0);

        let events = [
            Event::TouchBegin {
                id: 0,
                position: (50.0, 50.0).into(),
            },
            Event::TouchMove {
                id: 0,
                position: (100.0, 50.0).into(),
                delta: (50.0, 0.0).into(),
            },
            Event::TouchEnd {
                id: 0,
                position: (100.0, 50.0).into(),
            },
        ];

        let mut actions = vec![];
        for event in &events {
            ui.process(event, &mut path, &mut ctx, &mut actions);
        }

        let vid = ctx.view_id(&path);
        assert!(ctx.state_map.contains_key(&vid));
        // State should have changed.
        assert_eq!(*s.get(&ctx), 0.125);
    }
}
