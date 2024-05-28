use crate::font::{write_ascii, write_string};
use crate::graphics::{Graphics, PixelColor};
extern crate alloc;
use alloc::string::String;

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
    _buffer: [[char; 80]; 25],
    _cursor_row: usize,
    _cursor_col: usize,
}

impl Console {
    const N_ROWS: usize = 25;
    const N_COLS: usize = 80;

    fn new(_g: Graphics, _fg_color: PixelColor, _bg_color: PixelColor) -> Self {
        let mut _buffer = [[' '; 80]; 25];
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
            panic!("Console is already initialized");
        }
        unsafe { IS_INITIALIZED = true };
        unsafe { core::ptr::write(CONSOLE.as_mut_ptr(), Console::new(g, fg_color, bg_color)) };
    }

    pub fn instance() -> &'static mut Console {
        if !unsafe { IS_INITIALIZED } {
            panic!("Console is not initialized");
        }
        unsafe { &mut *CONSOLE.as_mut_ptr() }
    }

    fn new_line(&mut self) -> () {
        self._cursor_col = 0;
        if self._cursor_row < self.n_rows - 1 {
            self._cursor_row += 1;
        } else {
            for row in 1..self.n_rows {
                for col in 0..self.n_cols {
                    self._buffer[row - 1][col] = self._buffer[row][col];
                }
                write_string(
                    &self._g,
                    0,
                    (row - 1) * 16,
                    &self._buffer[row - 1].iter().collect::<String>(),
                    &self._fg_color,
                );
            }
            for col in 0..self.n_cols {
                self._buffer[self.n_rows - 1][col] = ' ';
            }
        }
    }

    pub fn put_string(&mut self, s: &str) -> () {
        let mut x = self._cursor_col * 8;
        let y = self._cursor_row * 16;
        for c in s.chars() {
            if c == '\n' {
                self.new_line();
            } else if self._cursor_col < self.n_cols {
                write_ascii(&self._g, x, y, c, &self._fg_color);
                self._buffer[self._cursor_row][self._cursor_col] = c;
                x += 8;
                self._cursor_col += 1;
            } else {
                self.new_line();
                write_ascii(&self._g, x, y, c, &self._fg_color);
                self._buffer[self._cursor_row][self._cursor_col] = c;
                x += 8;
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
