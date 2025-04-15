mod calculator;
use calculator::Calculator;

use vui::*;

fn main() {
    Calculator::new()
        .dark_mode()
        // .rounded_corners()
        .show()
        // .size([300.0, 400.0])
        .run();
}
