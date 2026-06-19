use core::fmt;
use noto_sans_mono_bitmap::get_raster_width;
use spin::Mutex;

use crate::{
    Color, FONT_HEIGHT, FONT_WEIGHT,
    graphics::{self, Framebuffer},
};

struct KWriter {
    fb: Framebuffer,
    x: usize,
    y: usize,
}

// We are only single core that's why it's safe.
unsafe impl Send for KWriter {}

static KWRITER: Mutex<Option<KWriter>> = Mutex::new(None);

const FONT_W: usize = get_raster_width(FONT_WEIGHT, FONT_HEIGHT);
const FONT_H: usize = 18; // 16px + 2px spacing

pub fn init(fb: &Framebuffer) {
    // Clone will be removed later.
    let fb = fb.clone();
    *KWRITER.lock() = Some(KWriter { fb, x: 0, y: 0 });
}

impl fmt::Write for KWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let width = self.fb.width;
        let height = self.fb.height;

        for ch in s.chars() {
            if ch == '\n' {
                self.x = 0;
                self.y += FONT_H;
                continue;
            }

            if self.y + FONT_H >= height {
                self.y = 0;
                graphics::clear_background(
                    &self.fb,
                    Color {
                        r: 0,g: 0,b: 0,},
                );
            }

            if self.x + FONT_W >= width {
                self.x = 0;
                self.y += FONT_H;
            }

            let mut buf = [0u8; 4];
            let s = ch.encode_utf8(&mut buf);
            graphics::draw_text(
                &self.fb,
                s,
                (self.x, self.y),
                Color {
                    r: 255,
                    g: 255,
                    b: 255,
                },
            );
            self.x += FONT_W;
        }

        Ok(())
    }
}

pub fn _kprint(args: fmt::Arguments) {
    use fmt::Write;

    if let Some(writer) = KWRITER.lock().as_mut() {
        writer.write_fmt(args).ok();
    }
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::kprintln::_kprint(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($($arg:tt)*) => {{
        $crate::kprint!($($arg)*);
        $crate::kprint!("\n");
    }};
}
