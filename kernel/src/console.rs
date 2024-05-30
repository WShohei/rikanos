use crate::font::write_ascii;
use crate::graphics::{Graphics, PixelColor};
use crate::graphics::Vector2D;
extern crate alloc;

use core::fmt::Write;
use core::mem::MaybeUninit;

static mut CONSOLE: MaybeUninit<Console> = MaybeUninit::uninit();
static mut IS_INITIALIZED: bool = false;

#[allow(dead_code)]
pub struct Console {
    pub n_rows: usize,
    pub n_cols: usize,

    _g: Graphics,
    _fg_color: PixelColor,
    _bg_color: PixelColor,
    _buffer: [[char; 81]; 25],
    _cursor_row: usize,
    _cursor_col: usize,
}

#[allow(dead_code)]
impl Console {
    const N_ROWS: usize = 25;
    const N_COLS: usize = 80;

    fn new(_g: Graphics, _fg_color: PixelColor, _bg_color: PixelColor) -> Self {
        let mut _buffer = [[' '; 81]; 25];
        let mut _cursor_row = 0;
        let mut _cursor_col = 0;

        Console {
            n_rows: Self::N_ROWS,
            n_cols: Self::N_COLS,

            _g,
            _fg_color,
            _bg_color,
            _buffer,
            _cursor_row,
            _cursor_col,
        }
    }

    pub fn initialize(g: Graphics, fg_color: PixelColor, bg_color: PixelColor) {
        if unsafe { IS_INITIALIZED } {
            return;
        }
        unsafe {
            IS_INITIALIZED = true;
            core::ptr::write(CONSOLE.as_mut_ptr(), Console::new(g, fg_color, bg_color));
            let console = &mut *CONSOLE.as_mut_ptr();
            let frame_width = console.frame_width();
            let frame_height = console.frame_height();
            console._g.fill_regtangle(Vector2D::<usize>::new(0, 0), Vector2D::<usize>::new(frame_width, frame_height - 50), &console._bg_color);
            console._g.fill_regtangle(Vector2D::<usize>::new(0, frame_height - 50), Vector2D::<usize>::new(frame_width, 50), &PixelColor::new(1, 8, 17));
            console._g.fill_regtangle(Vector2D::<usize>::new(0, frame_height - 50), Vector2D::<usize>::new(frame_width / 5, 50), &PixelColor::new(80, 80, 80));
            console._g.fill_regtangle(Vector2D::<usize>::new(10, frame_height - 40), Vector2D::<usize>::new(30, 30), &PixelColor::new(160, 160, 160));
        }
    }

    pub fn instance() -> &'static mut Console {
        if !unsafe { IS_INITIALIZED } {
            panic!("Console is not initialized");
        }
        unsafe { &mut *CONSOLE.as_mut_ptr() }
    }

    fn frame_width(&self) -> usize {
        self._g.width()
    }

    fn frame_height(&self) -> usize {
        self._g.height()
    }

    fn clear(&mut self) -> () {
        self._g.clear(&self._bg_color);
    }

    fn new_line(&mut self) -> () {
        self._cursor_col = 0;
        if self._cursor_row < self.n_rows - 1 {
            self._cursor_row += 1;
        } else {
            self.clear();
            for row in 1..self.n_rows {
                for col in 0..self.n_cols + 1 {
                    self._buffer[row - 1][col] = self._buffer[row][col];
                    write_ascii(
                        &self._g,
                        col * 8,
                        (row - 1) * 16,
                        self._buffer[row - 1][col],
                        &self._fg_color,
                    );
                }
            }
            for col in 0..self.n_cols + 1 {
                self._buffer[self.n_rows - 1][col] = ' ';
                write_ascii(
                    &self._g,
                    col * 8,
                    (self.n_rows - 1) * 16,
                    ' ',
                    &self._fg_color,
                );
            }
        }
    }

    fn put_string(&mut self, s: &str) -> () {
        for c in s.chars() {
            if c == '\n' {
                self.new_line();
            } else if self._cursor_col < self.n_cols - 1 {
                write_ascii(
                    &self._g,
                    self._cursor_col * 8,
                    self._cursor_row * 16,
                    c,
                    &self._fg_color,
                );
                self._buffer[self._cursor_row][self._cursor_col] = c;
                self._cursor_col += 1;
            } else {
                self.new_line();
                write_ascii(
                    &self._g,
                    self._cursor_col * 8,
                    self._cursor_row * 16,
                    c,
                    &self._fg_color,
                );
                self._buffer[self._cursor_row][self._cursor_col] = c;
                self._cursor_col += 1;
            }
        }
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.put_string(s);
        Ok(())
    }
}
