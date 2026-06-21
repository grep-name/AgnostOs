use uefi::Status;

use crate::{
    console,
    keyboard::{self, KeyboardEvent},
    kprint, kprintln,
};

pub fn init() -> Status {
    let mut line = String::new();

    kprint!("> ");
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
                        kprint!("> ");
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
                    kprint!("> ");
                }
                KeyboardEvent::ZoomIn => {
                    console::zoom_in();
                }
                KeyboardEvent::ZoomOut => {
                    console::zoom_out();
                }
            }

            console::draw_cursor(); // redraw cursor at new position
        }
    }
}

fn run_command(command: &str) {
    let command = command.trim();

    match command {
        "help" => kprintln!("Commands: help, clear, about, history"),
        "about" => kprintln!("AgnostOs v0.1 - written in Rust \n github.com/grep-name/agnostos"),
        "history" => {
            console::print_history();
        }
        "" => {}
        "clear" => console::reset(),
        _ => {
            kprintln!("Unknown command");
        }
    }
}
