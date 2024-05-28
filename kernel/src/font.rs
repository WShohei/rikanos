use crate::ascii_font::FONTS;
use crate::graphics::{Graphics, PixelColor};

pub fn write_ascii(g: &Graphics, x: usize, y: usize, c: char, color: &PixelColor) -> () {
    if c as usize >= FONTS.len() {
        return;
    }
    let font = FONTS[c as usize];
    for dy in 0..16 {
        for dx in 0..8 {
            if (font[dy] << dx) & 0x80 != 0 {
                g.write_pixel(x + dx, y + dy, &color);
            }
        }
    }
}

pub fn write_string(g: &Graphics, x: usize, y: usize, s: &str, color: &PixelColor) -> () {
    let mut x = x;
    for c in s.chars() {
        write_ascii(g, x, y, c, color);
        x += 8;
    }
}
