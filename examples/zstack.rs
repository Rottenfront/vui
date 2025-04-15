use vui::*;

fn main() {
    vui(zstack((
        "This is a test.",
        circle().color(RED_HIGHLIGHT).padding(Auto),
    )));
}
