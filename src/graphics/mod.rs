use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};

#[derive(Debug)]
pub struct Framebuffer {
    pub ptr: *mut u8,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub is_bgr: bool, // true if pixel format is BGR, false if RGB
}

impl Framebuffer {
    pub fn new(gop: &mut GraphicsOutput) -> Self {
        let mode_info = gop.current_mode_info();
        let (width, height) = mode_info.resolution();
        let stride = mode_info.stride();
        let is_bgr = matches!(mode_info.pixel_format(), PixelFormat::Bgr);
        let ptr = gop.frame_buffer().as_mut_ptr();

        Self {
            ptr,
            width,
            height,
            stride,
            is_bgr,
        }
    }
}

impl Framebuffer {
    #[inline]
    unsafe fn write_pixel(&self, pixel_index: usize, color: [u8; 3]) {
        let rgb = if self.is_bgr {
            [color[2], color[1], color[0]]
        } else {
            color
        };
        let p = unsafe { self.ptr.add(4 * pixel_index) };
        unsafe {
            p.write(rgb[0]);
            p.add(1).write(rgb[1]);
            p.add(2).write(rgb[2]);
        }
    }
}

pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 8;

/// Clears the background with the given color
///
/// **Example**
///
/// ```rust
/// something::graphics::clear_background(&fb, [255, 255, 255]);
/// ```
pub fn clear_background(fb: &Framebuffer, color: [u8; 3]) {
    for row in 0..fb.height {
        for col in 0..fb.width {
            let pixel_index = row * fb.stride + col;
            unsafe { fb.write_pixel(pixel_index, color) };
        }
    }
}

/// Renders a rectangle on the screen, at the provided coordinates with the provided color and
/// dimensions.
///
/// **Example**
///
/// ```rust
/// something::graphics::draw_rec(&fb, (100, 100), (100, 100), [0, 0, 0]);
/// ```
pub fn draw_rec(fb: &Framebuffer, (x, y): (usize, usize), (w, h): (usize, usize), color: [u8; 3]) {
    let x2 = x + w;
    let y2 = y + h;
    assert!(x2 <= fb.width, "Bad X coordinate");
    assert!(y2 <= fb.height, "Bad Y coordinate");

    for row in y..y2 {
        for col in x..x2 {
            let pixel_index = row * fb.stride + col;
            unsafe { fb.write_pixel(pixel_index, color) };
        }
    }
}

/// Renders a circle on the screen, at the provided coordinates with the provided color and radius.
///
/// **Example**
///
/// ```rust
/// something::graphics::draw_circle(&fb, 20, (100, 100), [0, 0, 0]);
/// ```
pub fn draw_circle(fb: &Framebuffer, radius: usize, (cx, cy): (usize, usize), color: [u8; 3]) {
    let r = radius as isize;
    let cx = cx as isize;
    let cy = cy as isize;
    let r_sq = r * r;

    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r_sq {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && py >= 0 {
                    let pixel_index = (py as usize) * fb.stride + (px as usize);
                    unsafe { fb.write_pixel(pixel_index, color) };
                }
            }
        }
    }
}

/// Renders a line on the screen, at the provided coordinates with the provided color.
///
/// **Example**
///
/// ```rust
/// something::graphics::draw_line(&fb, (100, 100), (100, 100), [0, 0, 0]);
/// ```
pub fn draw_line(fb: &Framebuffer, (x1, y1): (i64, i64), (x2, y2): (i64, i64), color: [u8; 3]) {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x2 >= x1 { 1 } else { -1 };
    let sy = if y2 >= y1 { 1 } else { -1 };
    let mut err = dx - dy;

    let (mut x, mut y) = (x1, y1);
    loop {
        let pixel_index = (y as usize) * fb.stride + (x as usize);
        unsafe { fb.write_pixel(pixel_index, color) };
        if x == x2 && y == y2 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

/// Renders the provided text on the screen, at the provided coordinates with the provided color and scale.
///
/// **Example**
///
/// ```rust
/// something::graphics::draw_text(&fb, "Random text to render", (100, 200), [0, 0, 0], 1);
/// ```
pub fn draw_text(
    fb: &Framebuffer,
    text: &str,
    (x, y): (usize, usize),
    color: [u8; 3],
    scale: usize,
) {
    let mut cursor_x = x;
    let cursor_y = y;

    for ch in text.chars() {
        let glyph = get_glyph(ch);

        for (row, byte) in glyph.iter().enumerate() {
            for col in 0..8 {
                if byte & (1 << col) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = cursor_x + col * scale + sx;
                            let py = cursor_y + row * scale + sy;
                            let pixel_index = py * fb.stride + px;
                            unsafe { fb.write_pixel(pixel_index, color) };
                        }
                    }
                }
            }
        }
        cursor_x += FONT_WIDTH * scale + scale;
    }
}

fn get_glyph(ch: char) -> [u8; 8] {
    use font8x8::legacy::BASIC_LEGACY;
    let idx = ch as usize;
    if idx < 128 {
        BASIC_LEGACY[idx]
    } else {
        BASIC_LEGACY[0]
    }
}
