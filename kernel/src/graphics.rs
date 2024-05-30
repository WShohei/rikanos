use uefi::proto::console::gop::ModeInfo;

use core::mem::MaybeUninit;

static mut GRAPHICS: MaybeUninit<Graphics> = MaybeUninit::uninit();
static mut IS_INITIALIZED: bool = false;

pub struct Vector2D<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2D<T> {
    pub fn new(x: T, y: T) -> Self {
        Vector2D { x, y }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FrameBufferConfig {
    pub frame_buffer: usize,
    pub frame_buffer_size: usize,
    pub mode_info: ModeInfo,
}

pub struct PixelColor {
    r: u8,
    g: u8,
    b: u8,
}

impl PixelColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        PixelColor { r, g, b }
    }
}

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub struct Graphics {
    _cfg: FrameBufferConfig,
    _pixel_writer: unsafe fn(&FrameBufferConfig, usize, usize, &PixelColor) -> (),
}

impl Graphics {
    fn new(cfg: FrameBufferConfig) -> Self {
        unsafe fn write_pixel_rgb(
            cfg: &FrameBufferConfig,
            x: usize,
            y: usize,
            color: &PixelColor,
        ) -> () {
            let addr = cfg.frame_buffer + (y * cfg.mode_info.stride() + x) * 4;
            core::ptr::write_volatile(
                addr as *mut u32,
                (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32,
            );
        }

        unsafe fn write_pixel_bgr(
            cfg: &FrameBufferConfig,
            x: usize,
            y: usize,
            color: &PixelColor,
        ) -> () {
            let addr = cfg.frame_buffer + (y * cfg.mode_info.stride() + x) * 4;
            core::ptr::write_volatile(
                addr as *mut u32,
                (color.b as u32) << 16 | (color.g as u32) << 8 | color.r as u32,
            );
        }

        let pixel_writer = match cfg.mode_info.pixel_format() {
            uefi::proto::console::gop::PixelFormat::Rgb => write_pixel_rgb,
            uefi::proto::console::gop::PixelFormat::Bgr => write_pixel_bgr,
            _ => panic!("unsupported pixel format"),
        };

        Graphics {
            _cfg: cfg,
            _pixel_writer: pixel_writer,
        }
    }

    pub fn initialize(cfg: FrameBufferConfig) -> () {
        if unsafe { IS_INITIALIZED } {
            return;
        }
        unsafe { IS_INITIALIZED = true };
        unsafe { core::ptr::write(GRAPHICS.as_mut_ptr(), Graphics::new(cfg)) };
    }

    pub fn instance() -> &'static Graphics {
        if !unsafe { IS_INITIALIZED } {
            panic!("Graphics is not initialized");
        }
        unsafe { &*GRAPHICS.as_ptr() }
    }

    pub fn write_pixel(&self, x: usize, y: usize, color: &PixelColor) -> () {
        unsafe {
            (self._pixel_writer)(&self._cfg, x, y, color);
        }
    }

    pub fn fill_regtangle(
        &self,
        pos: Vector2D<usize>,
        size: Vector2D<usize>,
        color: &PixelColor,
    ) -> () {
        for y in pos.y..pos.y + size.y {
            for x in pos.x..pos.x + size.x {
                self.write_pixel(x, y, color);
            }
        }
    }

    pub fn clear(&self, color: &PixelColor) -> () {
        for y in 0..self.height() {
            for x in 0..self.width() {
                self.write_pixel(x, y, color);
            }
        }
    }

    pub fn width(&self) -> usize {
        self._cfg.mode_info.resolution().0 as usize
    }

    pub fn height(&self) -> usize {
        self._cfg.mode_info.resolution().1 as usize
    }
}
