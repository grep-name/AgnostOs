use core::fmt;
use noto_sans_mono_bitmap::{RasterHeight, get_raster_width};
use spin::Mutex;

use crate::{
    FONT_WEIGHT, color,
    graphics::{self, Framebuffer},
};

struct KWriter {
    fb: Framebuffer,
    x: usize,
    y: usize,
    history: Vec<String>,
    current_line: String,
    font_size: RasterHeight,
}

unsafe impl Send for KWriter {}

static KWRITER: Mutex<Option<KWriter>> = Mutex::new(None);

fn font_h(size: RasterHeight) -> usize {
    let px = match size {
        RasterHeight::Size16 => 16,
        RasterHeight::Size20 => 20,
        RasterHeight::Size24 => 24,
        RasterHeight::Size32 => 32,
    };
    px + 2 // spacing
}

fn font_w(size: RasterHeight) -> usize {
    get_raster_width(FONT_WEIGHT, size)
}

pub fn init(fb: &Framebuffer) {
    let fb = fb.clone();
    *KWRITER.lock() = Some(KWriter {
        fb,
        x: 0,
        y: 0,
        font_size: RasterHeight::Size16,
        current_line: String::new(),
        history: Vec::new(),
    });
}

impl fmt::Write for KWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let width = self.fb.width;
        let height = self.fb.height;
        let fh = font_h(self.font_size);
        let fw = font_w(self.font_size);

        for ch in s.chars() {
            if ch == '\n' {
                self.history.push(core::mem::take(&mut self.current_line));
                self.x = 0;
                self.y += fh;
                continue;
            }

            self.current_line.push(ch);

            if self.y + fh >= height {
                self.y = 0;
                graphics::clear_background(&self.fb, color::BLACK);
            }

            if self.x + fw >= width {
                self.x = 0;
                self.y += fh;
            }

            let mut buf = [0u8; 4];
            let s = ch.encode_utf8(&mut buf);
            graphics::draw_text(
                &self.fb,
                s,
                (self.x, self.y),
                color::WHITE,
                Some(self.font_size),
            );
            self.x += fw;
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
        $crate::console::_kprint(format_args!($($arg)*))
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
        writer.history.clear();
        writer.current_line.clear();
    }
}

pub fn backspace() {
    if let Some(writer) = KWRITER.lock().as_mut() {
        let fh = font_h(writer.font_size);
        let fw = font_w(writer.font_size);

        if writer.x == 0 {
            if writer.y == 0 {
                return;
            }
            writer.y -= fh;
            writer.x = writer.fb.width - (writer.fb.width % fw) - fw;
        } else {
            writer.x -= fw;
        }

        crate::graphics::draw_rec(
            &writer.fb,
            (writer.x, writer.y),
            (fw, fh),
            crate::color::BLACK,
        );
    }
}

pub fn draw_cursor() {
    if let Some(writer) = KWRITER.lock().as_mut() {
        let fh = font_h(writer.font_size);
        let fw = font_w(writer.font_size);
        crate::graphics::draw_rec(
            &writer.fb,
            (writer.x, writer.y),
            (fw, fh - 4),
            crate::color::WHITE,
        );
    }
}

pub fn erase_cursor() {
    if let Some(writer) = KWRITER.lock().as_mut() {
        let fh = font_h(writer.font_size);
        let fw = font_w(writer.font_size);
        crate::graphics::draw_rec(
            &writer.fb,
            (writer.x, writer.y),
            (fw, fh - 4),
            crate::color::BLACK,
        );
    }
}

pub fn print_history() {
    if let Some(writer) = KWRITER.lock().as_mut() {
        let fh = font_h(writer.font_size);
        let mut y = 0;

        for line in writer.history.iter() {
            graphics::draw_text(
                &writer.fb,
                line,
                (0, y),
                color::WHITE,
                Some(writer.font_size),
            );
            y += fh;
        }

        writer.x = 0;
        writer.y = y;
    } else {
        kprintln!("Failed to print history");
    }
}

pub fn zoom_in() {
    if let Some(writer) = KWRITER.lock().as_mut() {
        writer.font_size = match writer.font_size {
            RasterHeight::Size16 => RasterHeight::Size20,
            RasterHeight::Size20 => RasterHeight::Size24,
            RasterHeight::Size24 => RasterHeight::Size32,
            RasterHeight::Size32 => RasterHeight::Size32,
        };
        graphics::clear_background(&writer.fb, color::BLACK);
    }
    print_history();
    kprint!("> ");
}

pub fn zoom_out() {
    if let Some(writer) = KWRITER.lock().as_mut() {
        writer.font_size = match writer.font_size {
            RasterHeight::Size32 => RasterHeight::Size24,
            RasterHeight::Size24 => RasterHeight::Size20,
            RasterHeight::Size20 => RasterHeight::Size16,
            RasterHeight::Size16 => RasterHeight::Size16,
        };
        graphics::clear_background(&writer.fb, color::BLACK);
    }
    print_history();
    kprint!("> ");
}
