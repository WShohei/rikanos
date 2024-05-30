use crate::graphics::Vector2D;
use crate::graphics::{Graphics, PixelColor};
use crate::{print, println};
use core::mem::MaybeUninit;

static mut MOUSE_CURSOR: MaybeUninit<MouseCursor> = MaybeUninit::uninit();
static mut IS_INITIALIZED: bool = false;

pub struct MouseCursor {
    _position: Vector2D<usize>,
    _graphics: Graphics,
}

impl MouseCursor {
    fn new(g: Graphics, init_pos: Vector2D<usize>) -> Self {
        MouseCursor {
            _position: init_pos,
            _graphics: g,
        }
    }

    pub fn initialize(g: Graphics, init_pos: Vector2D<usize>) {
        if unsafe { IS_INITIALIZED } {
            return;
        }
        unsafe {
            IS_INITIALIZED = true;
            core::ptr::write(MOUSE_CURSOR.as_mut_ptr(), MouseCursor::new(g, init_pos));
            let mouse_cursor = &mut *MOUSE_CURSOR.as_mut_ptr();
            mouse_cursor.draw();
        }
    }

    pub fn instance() -> &'static mut MouseCursor {
        if !unsafe { IS_INITIALIZED } {
            println!("MouseCursor is not initialized");
            panic!();
        }
        unsafe { &mut *MOUSE_CURSOR.as_mut_ptr() }
    }

    pub fn draw(&self) {
        let g = &self._graphics;
        let x = self._position.x;
        let y = self._position.y;
        for (dy, &line) in MOUSE_CURSOR_SHAPE.iter().enumerate() {
            for (dx, c) in line.chars().enumerate() {
                if c == '@' {
                    g.write_pixel(x + dx, y + dy, &PixelColor::new(0, 0, 0));
                } else if c == '.' {
                    g.write_pixel(x + dx, y + dy, &PixelColor::new(255, 255, 255));
                }
            }
        }
    }

    pub fn erase(&self) {
        let g = &self._graphics;
        let x = self._position.x;
        let y = self._position.y;
        for dy in 0..MOUSE_CURSOR_SHAPE.len() {
            for dx in 0..MOUSE_CURSOR_SHAPE[dy].len() {
                g.write_pixel(x + dx, y + dy, &PixelColor::new(0, 0, 0));
            }
        }
    }

    pub fn move_relative(&mut self, displacement_x: i8, displacement_y: i8) {
        self.erase();
        let x = self._position.x as i8 + displacement_x;
        let y = self._position.y as i8 + displacement_y;
        self._position.x = x as usize;
        self._position.y = y as usize;
        self.draw();
    }
}

const MOUSE_CURSOR_SHAPE: [&str; 24] = [
    "@              ",
    "@@             ",
    "@.@            ",
    "@..@           ",
    "@...@          ",
    "@....@         ",
    "@.....@        ",
    "@......@       ",
    "@.......@      ",
    "@........@     ",
    "@.........@    ",
    "@..........@   ",
    "@...........@  ",
    "@............@ ",
    "@......@@@@@@@@",
    "@......@       ",
    "@....@@.@      ",
    "@...@ @.@      ",
    "@..@   @.@     ",
    "@.@    @.@     ",
    "@@      @.@    ",
    "@       @.@    ",
    "         @.@   ",
    "         @@@   ",
];
