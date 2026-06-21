use core::arch::asm;

pub unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe {
        asm!("in al, dx", out("al") val, in("dx") port);
    }
    val
}

pub fn read_scan_code_if_available() -> Option<u8> {
    unsafe {
        let status = inb(0x64);
        if status & 1 != 0 {
            return Some(inb(0x60));
        } else {
            return None;
        }
    }
}

// minimal US QWERTY scancode set 1 -> ascii
pub fn scancode_to_ascii(code: u8) -> Option<char> {
    // ignore key release (top bit set)
    if code & 0x80 != 0 {
        return None;
    }
    let c = match code {
        0x1E => 'a',
        0x30 => 'b',
        0x2E => 'c',
        0x20 => 'd',
        0x12 => 'e',
        0x21 => 'f',
        0x22 => 'g',
        0x23 => 'h',
        0x17 => 'i',
        0x24 => 'j',
        0x25 => 'k',
        0x26 => 'l',
        0x32 => 'm',
        0x31 => 'n',
        0x18 => 'o',
        0x19 => 'p',
        0x10 => 'q',
        0x13 => 'r',
        0x1F => 's',
        0x14 => 't',
        0x16 => 'u',
        0x2F => 'v',
        0x11 => 'w',
        0x2D => 'x',
        0x15 => 'y',
        0x2C => 'z',
        0x02 => '1',
        0x03 => '2',
        0x04 => '3',
        0x05 => '4',
        0x06 => '5',
        0x07 => '6',
        0x08 => '7',
        0x09 => '8',
        0x0A => '9',
        0x0B => '0',
        0x39 => ' ',
        0x1C => '\n',
        0x0E => '\u{8}', // backspace
        _ => return None,
    };
    Some(c)
}
