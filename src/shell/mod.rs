use core::sync::atomic::Ordering;

use noto_sans_mono_bitmap::RasterHeight;
use uefi::Status;

use crate::{
    HEAP_SIZE, HEAP_START, PROMPT, color, console,
    graphics::{self, Framebuffer},
    keyboard::{self, KeyboardEvent},
    kprint, kprintln,
};

pub fn init(fb: &Framebuffer) -> Status {
    graphics::clear_background(&fb, color::BLACK);
    let mut line = String::new();

    kprint!("{PROMPT}");
    console::draw_cursor(); // draw cursor at the very beginning

    loop {
        if let Some(key) = keyboard::poll() {
            console::erase_cursor();
            match key {
                KeyboardEvent::Char(c) => match c {
                    '\n' => {
                        kprintln!();
                        run_command(&line);
                        line.clear();
                        kprint!("{PROMPT}");
                    }
                    '\u{8}' => {
                        if line.pop().is_some() {
                            console::backspace();
                        }
                    }
                    c => {
                        line.push(c);
                        kprint!("{}", c);
                    }
                },
                KeyboardEvent::CtrlC => {
                    kprintln!("^C");
                    line.clear();
                    kprint!("{PROMPT}");
                }
                KeyboardEvent::ZoomIn => {
                    console::zoom_in();
                }
                KeyboardEvent::ZoomOut => {
                    console::zoom_out();
                }
                KeyboardEvent::ArrowUp => {
                    console::arrow_up(&mut line);
                }
                KeyboardEvent::ArrowDown => {
                    console::arrow_down(&mut line);
                }
                KeyboardEvent::CtrlL => {
                    console::reset();
                    line.clear();
                    kprint!("{PROMPT}");
                }
            }

            console::draw_cursor(); // redraw cursor at new position
        }
    }
}

fn run_command(command: &str) {
    let command = command.trim();

    let mut iter = command.split_whitespace();

    let command = iter.next().unwrap_or("");
    let mut flags = Vec::new();
    let mut args = Vec::new();

    for part in iter {
        if part.starts_with('-') {
            flags.push(part);
        } else {
            args.push(part);
        }
    }

    match command {
        "help" => help(args),
        "about" => kprintln!("AgnostOs v0.1 - written in Rust \n github.com/grep-name/agnostos"),
        "history" => {
            console::print_history();
        }
        "echo" => {
            kprintln!("{}", args.join(" "));
        }
        "meminfo" => {
            let start = HEAP_START.load(Ordering::Relaxed);
            let size = HEAP_SIZE.load(Ordering::Relaxed);

            kprintln!("heap start: {:#x}", start);
            kprintln!("heap size:  {}mb", size / (1024 * 1024));
        }
        "font" => match args.first().copied().unwrap_or("") {
            "16" => console::set_font_size(RasterHeight::Size16),
            "20" => console::set_font_size(RasterHeight::Size20),
            "24" => console::set_font_size(RasterHeight::Size24),
            "32" => console::set_font_size(RasterHeight::Size32),
            _ => kprintln!("usage: font <16|20|24|32>"),
        },
        "clear" => console::reset(),
        "" => {}
        _ => {
            kprintln!("Unknown command");
        }
    }
}

fn help(args: Vec<&str>) {
    if let Some(cmd) = args.first() {
        match *cmd {
            "help" => {
                kprintln!("help - show available commands");
                kprintln!("usage: help [command]");
                kprintln!("example: help echo");
            }
            "echo" => {
                kprintln!("echo - print text to the screen");
                kprintln!("usage: echo <text>");
                kprintln!("example: echo hello world");
            }
            "clear" => {
                kprintln!("clear - clear the screen and reset cursor");
                kprintln!("usage: clear");
            }
            "about" => {
                kprintln!("about - show information about AgnostOs");
                kprintln!("usage: about");
            }
            "history" => {
                kprintln!("history - print everything on screen from the beginning");
                kprintln!("usage: history");
            }
            "font" => {
                kprintln!("font - change the font size");
                kprintln!("usage: font <16|20|24|32>");
                kprintln!("example: font 24");
            }
            "meminfo" => {
                kprintln!("meminfo - show memory information");
                kprintln!("usage: meminfo");
            }
            _ => kprintln!("unknown command: {}", cmd),
        }
    } else {
        kprintln!("AgnostOs shell - available commands:");
        kprintln!("");
        kprintln!("  help      show this message, or help for a specific command");
        kprintln!("  echo      print text to the screen");
        kprintln!("  clear     clear the screen");
        kprintln!("  about     show OS information");
        kprintln!("  history   print screen history");
        kprintln!("  font      change font size");
        kprintln!("  meminfo   show heap memory information");
        kprintln!("");
        kprintln!("tip: type 'help <command>' for more details");
        kprintln!("tip: ctrl+c to cancel, ctrl+plus/minus to zoom");
    }
}
