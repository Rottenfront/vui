use rui::*;

// This example shows a modifier that has state for
// managing an animation.

struct MyState {
    offset: f32,
}

impl Default for MyState {
    fn default() -> Self {
        MyState { offset: 100.0 }
    }
}

trait MyMod: View + Sized {
    fn my_modifier(self) -> impl View {
        state(MyState::default, move |s, cx| {
            self.clone()
                .offset(LocalOffset::new(0.0, cx[s].offset))
                .anim(move |cx, _| {
                    cx[s].offset *= 0.9;
                })
        })
    }
}

impl<V: View> MyMod for V {}

fn my_text(name: &str) -> impl View {
    let name = name.to_string();
    text(name.as_str())
}

fn main() {
    hstack((my_text("without"), my_text("with").my_modifier())).run()
}
