use crate::{FONT_HEIGHT, FONT_WEIGHT, color::Color};
use noto_sans_mono_bitmap::get_raster;
use uefi::{
    boot::{self, OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol},
    proto::console::gop::{BltOp, BltPixel, FrameBuffer, GraphicsOutput, PixelFormat},
};

type PixelWriter = unsafe fn(&mut FrameBuffer, usize, &Color);

unsafe fn write_pixel_rgb(fb: &mut FrameBuffer, pixel_base: usize, color: &Color) {
    unsafe { fb.write_value(pixel_base, [color.r, color.g, color.b]) }
}

unsafe fn write_pixel_bgr(fb: &mut FrameBuffer, pixel_base: usize, color: &Color) {
    unsafe { fb.write_value(pixel_base, [color.b, color.g, color.r]) }
}

/// Clears the background with the given color
///
/// **Example**
///
/// ```rust
/// use agnostos::Color;
///
/// agnostos::uefi_graphics::clear_background(gop, Color: { r: 255, g: 255, b: 255 });
/// ```
pub fn clear_background(gop: &mut GraphicsOutput, color: Color) {
    let (width, height) = gop.current_mode_info().resolution();
    let op = BltOp::VideoFill {
        color: BltPixel::new(color.r, color.g, color.b),
        dest: (0, 0),
        dims: (width, height),
    };

    gop.blt(op).expect("Failed to fill screen with color");
}

pub fn init_gop() -> ScopedProtocol<GraphicsOutput> {
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()
        .expect("missing graphics output protocol");

    let mut gop = unsafe {
        boot::open_protocol::<GraphicsOutput>(
            OpenProtocolParams {
                handle: gop_handle,
                agent: boot::image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
        .expect("failed to open Graphics Output Protocol")
    };

    let mode = gop
        .modes()
        .filter(|mode| {
            let (w, h) = mode.info().resolution();
            w <= 1920 && h <= 1080
        })
        .max_by_key(|mode| {
            let (w, h) = mode.info().resolution();
            w * h
        })
        .expect("no graphics modes available");

    gop.set_mode(&mode).expect("failed to set GOP mode");
    gop // return owned, not a reference
}

/// Renders a rectangle on the screen, at the provided coordinates with the provided color and
/// dimensions.
///
/// **Example**
///
/// ```rust
/// use agnostos::Color;
///
/// agnostos::uefi_graphics::draw_rec(gop, (100, 100), (100, 100), Color { r: 0, g: 0, b: 0 });
/// ```
pub fn draw_rec(
    gop: &mut GraphicsOutput,
    (x, y): (usize, usize),
    (w, h): (usize, usize),
    color: Color,
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
                write_pixel(&mut fb, pixel_base, &color);
            }
        }
    }
}

/// Renders a circle on the screen, at the provided coordinates with the provided color and radius.
///
/// **Example**
///
/// ```rust
/// use agnostos::Color;
///
/// agnostos::uefi_graphics::draw_circle(gop, 20, (100, 100), Color { r: 0, g: 0, b: 0 });
/// ```
pub fn draw_circle(
    gop: &mut GraphicsOutput,
    radius: usize,
    (cx, cy): (usize, usize),
    color: Color,
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
                        write_pixel(&mut fb, pixel_base, &color);
                    }
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
/// use agnostos::Color;
///
/// agnostos::uefi_graphics::draw_line(gop, (100, 100), (100, 100), Color { r: 0, g: 0, b: 0 });
/// ```
pub fn draw_line(
    gop: &mut GraphicsOutput,
    (x1, y1): (i64, i64),
    (x2, y2): (i64, i64),
    color: Color,
) {
    let mi = gop.current_mode_info();
    let stride = mi.stride();
    let mut fb = gop.frame_buffer();

    let pixel = match mi.pixel_format() {
        PixelFormat::Rgb => [color.r, color.g, color.b],
        PixelFormat::Bgr => [color.g, color.g, color.r],
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

/// Renders the provided text on the screen, at the provided coordinates with the provided color and scale.
///
/// **Example**
///
/// ```rust
/// use agnostos::Color;
///
/// agnostos::uefi_graphics::draw_text(gop, "Random text to render", (100, 200), Color { r: 0, g: 0, b: 0 });
/// ```
pub fn draw_text(gop: &mut GraphicsOutput, text: &str, (x, y): (usize, usize), color: Color) {
    let mi = gop.current_mode_info();
    let stride = mi.stride();
    let (width, height) = mi.resolution();
    let mut fb = gop.frame_buffer();

    let write_pixel: PixelWriter = match mi.pixel_format() {
        PixelFormat::Rgb => write_pixel_rgb,
        PixelFormat::Bgr => write_pixel_bgr,
        _ => return,
    };

    let mut cursor_x = x;

    for ch in text.chars() {
        let char_raster = match get_raster(ch, FONT_WEIGHT, FONT_HEIGHT) {
            Some(r) => r,
            None => match get_raster('?', FONT_WEIGHT, FONT_HEIGHT) {
                Some(r) => r,
                None => continue,
            },
        };

        for (row, row_data) in char_raster.raster().iter().enumerate() {
            for (col, &intensity) in row_data.iter().enumerate() {
                if intensity == 0 {
                    continue;
                }

                let px = cursor_x + col;
                let py = y + row;

                if px >= width || py >= height {
                    continue;
                }

                let r = (color.r as u32 * intensity as u32 / 255) as u8;
                let g = (color.g as u32 * intensity as u32 / 255) as u8;
                let b = (color.b as u32 * intensity as u32 / 255) as u8;

                unsafe {
                    let pixel_index = py * stride + px;
                    write_pixel(&mut fb, 4 * pixel_index, &Color::new(r, g, b));
                }
            }
        }

        cursor_x += char_raster.width();
    }
}
