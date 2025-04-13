use crate::*;
use peniko::Color;

const fn hex_digit(x: u8) -> u8 {
    (if x >= b'0' && x <= b'9' {
        x - b'0'
    } else if x >= b'a' && x <= b'f' {
        x - b'a' + 10
    } else if x >= b'A' && x <= b'F' {
        x - b'A' + 10
    } else {
        panic!("bad hex digit")
    }) as u8
}

const fn hex_const(s: &str) -> Color {
    let s = s.as_bytes();
    Color::from_rgb8(
        hex_digit(s[1]) * 16 + hex_digit(s[2]),
        hex_digit(s[3]) * 16 + hex_digit(s[4]),
        hex_digit(s[5]) * 16 + hex_digit(s[6]),
    )
}

pub const TEXT_COLOR: Color = hex_const("#D6D6D6");
pub const RED_HIGHLIGHT: Color = hex_const("#FF0062");
pub const RED_HIGHLIGHT_DARK: Color = hex_const("#A60040");
pub const RED_HIGHLIGHT_BACKGROUND: Color = hex_const("#1C000B");
pub const AZURE_HIGHLIGHT: Color = hex_const("#00D4FF");
pub const AZURE_HIGHLIGHT_DARK: Color = hex_const("#009BBA");
pub const AZURE_HIGHLIGHT_BACKGROUND: Color = hex_const("#000F14");
pub const GREEN_HIGHLIGHT: Color = hex_const("#3BC455");

pub const BUTTON_BACKGROUND_COLOR: Color = Color::from_rgba8(25, 25, 25, 255);

pub const BUTTON_HOVER_COLOR: Color = Color::from_rgba8(50, 50, 50, 255);

pub const BUTTON_DOWN_COLOR: Color = Color::from_rgba8(13, 13, 13, 255);

pub const CLEAR_COLOR: Color = Color::from_rgba8(0, 0, 0, 0);

pub const WHITE: Color = Color::from_rgba8(255, 255, 255, 255);

pub const BLACK: Color = Color::from_rgba8(0, 0, 0, 255);

pub const CONTROL_BACKGROUND: Color = Color::from_rgba8(35, 35, 35, 255);
pub const MEDIUM_GRAY: Color = Color::from_rgba8(136, 136, 136, 255);

pub const GROOVES: Color = hex_const("#252A2B");
pub const GROOVES_DARK: Color = hex_const("#0D0D0D");
