use font8x8::legacy::BASIC_LEGACY;
use uefi::proto::console::gop::{BltOp, BltPixel, FrameBuffer, GraphicsOutput, PixelFormat};

pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 8;

type PixelWriter = unsafe fn(&mut FrameBuffer, usize, [u8; 3]);

unsafe fn write_pixel_rgb(fb: &mut FrameBuffer, pixel_base: usize, rgb: [u8; 3]) {
    unsafe { fb.write_value(pixel_base, rgb) }
}
unsafe fn write_pixel_bgr(fb: &mut FrameBuffer, pixel_base: usize, rgb: [u8; 3]) {
    unsafe { fb.write_value(pixel_base, [rgb[2], rgb[1], rgb[0]]) }
}

pub fn draw_rec(
    gop: &mut GraphicsOutput,
    (x, y): (usize, usize),
    (w, h): (usize, usize),
    color: [u8; 3],
) {
    let mi = gop.current_mode_info();
    let stride = mi.stride();
    let (width, height) = mi.resolution();
    let mut fb = gop.frame_buffer();

    let write_pixel: PixelWriter = match mi.pixel_format() {
        PixelFormat::Rgb => write_pixel_rgb,
        PixelFormat::Bgr => write_pixel_bgr,
        _ => return,
    };

    let x2 = x + w;
    let y2 = y + h;

    assert!((x < width) && (x2 <= width), "Bad X coordinate");
    assert!((y < height) && (y2 <= height), "Bad Y coordinate");

    for row in y..y2 {
        for column in x..x2 {
            unsafe {
                let pixel_index = (row * stride) + column;
                let pixel_base = 4 * pixel_index;
                write_pixel(&mut fb, pixel_base, color);
            }
        }
    }
}

pub fn clear_background(gop: &mut GraphicsOutput, color: [u8; 3]) {
    let (width, height) = gop.current_mode_info().resolution();
    let op = BltOp::VideoFill {
        color: BltPixel::new(color[0], color[1], color[2]),
        dest: (0, 0),
        dims: (width, height),
    };

    gop.blt(op).expect("Failed to fill screen with color");
}

pub fn draw_line(
    gop: &mut GraphicsOutput,
    (x1, y1): (i64, i64),
    (x2, y2): (i64, i64),
    color: [u8; 3],
) {
    let mi = gop.current_mode_info();
    let stride = mi.stride();
    let mut fb = gop.frame_buffer();

    let pixel = match mi.pixel_format() {
        PixelFormat::Rgb => color,
        PixelFormat::Bgr => [color[2], color[1], color[0]],
        _ => return,
    };

    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x2 >= x1 { 1 } else { -1 };
    let sy = if y2 >= y1 { 1 } else { -1 };
    let mut err = dx - dy;

    let (mut x, mut y) = (x1, y1);
    loop {
        unsafe {
            let pixel_index = (y as usize * stride) + x as usize;
            fb.write_value(4 * pixel_index, pixel);
        }
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

pub fn draw_circle(
    gop: &mut GraphicsOutput,
    radius: usize,
    (cx, cy): (usize, usize),
    color: [u8; 3],
) {
    let mi = gop.current_mode_info();
    let stride = mi.stride();
    let mut fb = gop.frame_buffer();

    let write_pixel: PixelWriter = match mi.pixel_format() {
        PixelFormat::Rgb => write_pixel_rgb,
        PixelFormat::Bgr => write_pixel_bgr,
        _ => return,
    };

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
                    unsafe {
                        let pixel_index = (py as usize * stride) + px as usize;
                        let pixel_base = 4 * pixel_index;
                        write_pixel(&mut fb, pixel_base, color);
                    }
                }
            }
        }
    }
}

pub fn draw_text(
    gop: &mut GraphicsOutput,
    text: &str,
    (x, y): (usize, usize),
    color: [u8; 3],
    scale: usize,
) {
    let mi = gop.current_mode_info();
    let stride = mi.stride();
    let mut fb = gop.frame_buffer();

    let write_pixel: PixelWriter = match mi.pixel_format() {
        PixelFormat::Rgb => write_pixel_rgb,
        PixelFormat::Bgr => write_pixel_bgr,
        _ => return,
    };

    let mut cursor_x = x;
    let cursor_y = y;

    for ch in text.chars() {
        let glyph = get_glyph(ch);

        for (row, byte) in glyph.iter().enumerate() {
            for col in 0..8 {
                if byte & (1 << col) != 0 {
                    // draw a scale x scale block for this bit
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = cursor_x + col * scale + sx;
                            let py = cursor_y + row * scale + sy;
                            unsafe {
                                let pixel_index = (py * stride) + px;
                                write_pixel(&mut fb, 4 * pixel_index, color);
                            }
                        }
                    }
                }
            }
        }
        cursor_x += FONT_WIDTH * scale + scale; // advance + 1px spacing
    }
}

fn get_glyph(ch: char) -> [u8; 8] {
    let idx = ch as usize;
    if idx < 128 {
        BASIC_LEGACY[idx]
    } else {
        BASIC_LEGACY[0]
    }
}
