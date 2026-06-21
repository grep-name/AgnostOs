use core::fmt;
use noto_sans_mono_bitmap::get_raster_width;
use spin::Mutex;

use crate::{
    FONT_HEIGHT, FONT_WEIGHT, color,
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
                graphics::clear_background(&self.fb, color::BLACK);
            }

            if self.x + FONT_W >= width {
                self.x = 0;
                self.y += FONT_H;
            }

            let mut buf = [0u8; 4];
            let s = ch.encode_utf8(&mut buf);
            graphics::draw_text(&self.fb, s, (self.x, self.y), color::WHITE);
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

pub fn reset() {
    if let Some(writer) = KWRITER.lock().as_mut() {
        graphics::clear_background(&writer.fb, color::BLACK);
        writer.x = 0;
        writer.y = 0;
    }
}

pub fn backspace() {
    if let Some(writer) = KWRITER.lock().as_mut() {
        if writer.x == 0 {
            if writer.y == 0 {
                return; // top-left corner, nothing to erase
            }
            // x == 0 and there's a previous line — wrap up to it
            writer.y -= FONT_H;
            writer.x = writer.fb.width - (writer.fb.width % FONT_W) - FONT_W;
        } else {
            // normal case: just move back within this line
            writer.x -= FONT_W;
        }

        crate::graphics::draw_rec(
            &writer.fb,
            (writer.x, writer.y),
            (FONT_W, FONT_H),
            crate::color::BLACK,
        );
    }
}

pub fn draw_cursor() {
    if let Some(writer) = KWRITER.lock().as_mut() {
        crate::graphics::draw_rec(
            &writer.fb,
            (writer.x, writer.y),
            (FONT_W, FONT_H - 4),
            crate::color::WHITE, // cursor color
        );
    }
}

pub fn erase_cursor() {
    if let Some(writer) = KWRITER.lock().as_mut() {
        crate::graphics::draw_rec(
            &writer.fb,
            (writer.x, writer.y),
            (FONT_W, FONT_H - 4),
            crate::color::BLACK, // background color
        );
    }
}
