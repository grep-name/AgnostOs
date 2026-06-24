use core::fmt;
use noto_sans_mono_bitmap::{RasterHeight, get_raster_width};
use spin::Mutex;

use crate::{
    FONT_WEIGHT, PROMPT, color,
    graphics::{self, Framebuffer},
};

struct KWriter {
    fb: Framebuffer,
    x: usize,
    y: usize,
    history: Vec<String>,
    current_line: String,
    font_size: RasterHeight,
    history_index: Option<usize>, // None = not currently browsing history
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
        history_index: None,
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

pub(crate) fn reset() {
    if let Some(writer) = KWRITER.lock().as_mut() {
        graphics::clear_background(&writer.fb, color::BLACK);
        writer.x = 0;
        writer.y = 0;
        writer.history.clear();
        writer.current_line.clear();
        writer.history_index = None;
    }
}

pub(crate) fn backspace() {
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

pub(crate) fn draw_cursor() {
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

pub(crate) fn erase_cursor() {
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

pub(crate) fn print_history() {
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

pub(crate) fn zoom_in() {
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
    kprint!("{PROMPT}");
}

pub(crate) fn zoom_out() {
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
    kprint!("{PROMPT}");
}

pub(crate) fn arrow_up(current_line: &mut String) {
    if let Some(writer) = KWRITER.lock().as_mut() {
        let commands: Vec<&String> = writer
            .history
            .iter()
            .filter(|line| line.starts_with(PROMPT) && line != &"^C" && line != &"")
            .collect();

        if commands.is_empty() {
            return;
        }

        let new_index = match writer.history_index {
            None => commands.len() - 1, // start from most recent
            Some(0) => 0,               // already at oldest, stay
            Some(i) => i - 1,
        };

        writer.history_index = Some(new_index);

        let recalled = commands[new_index].trim_start_matches(PROMPT).to_string();

        redraw_input_line(writer, &recalled);
        current_line.clear();
        current_line.push_str(&recalled);
    }
}

pub(crate) fn arrow_down(current_line: &mut String) {
    if let Some(writer) = KWRITER.lock().as_mut() {
        let commands: Vec<&String> = writer
            .history
            .iter()
            .filter(|line| line.starts_with(PROMPT) && line != &"^C" && line != &"")
            .collect();

        if commands.is_empty() {
            return;
        }

        let new_text = match writer.history_index {
            None => return, // not browsing, nothing to do
            Some(i) if i + 1 < commands.len() => {
                writer.history_index = Some(i + 1);
                commands[i + 1].trim_start_matches(PROMPT).to_string()
            }
            Some(_) => {
                writer.history_index = None; // past the newest, clear back to empty input
                String::new()
            }
        };

        redraw_input_line(writer, &new_text);
        current_line.clear();
        current_line.push_str(&new_text);
    }
}

fn redraw_input_line(writer: &mut KWriter, text: &str) {
    let fh = font_h(writer.font_size);
    let fw = font_w(writer.font_size);

    // erase the current input line area (just this row, from x=0 to screen width)
    graphics::draw_rec(
        &writer.fb,
        (0, writer.y),
        (writer.fb.width, fh),
        color::BLACK,
    );

    graphics::draw_text(
        &writer.fb,
        PROMPT,
        (0, writer.y),
        color::WHITE,
        Some(writer.font_size),
    );

    graphics::draw_text(
        &writer.fb,
        text,
        (fw * PROMPT.chars().count(), writer.y),
        color::WHITE,
        Some(writer.font_size),
    );

    writer.x = fw * (PROMPT.chars().count() + text.chars().count());
}

pub(crate) fn set_font_size(font_size: RasterHeight) {
    if let Some(writer) = KWRITER.lock().as_mut() {
        writer.font_size = font_size;
    }
}
