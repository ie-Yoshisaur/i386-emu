use crate::emulator::{Emulator, Register8};
use crate::io::io_out8;

static BIOS_TO_TERMINAL: [i32; 8] = [30, 34, 32, 36, 31, 35, 33, 37];

fn put_string(s: &str) {
    for byte in s.bytes() {
        io_out8(0x03f8, byte);
    }
}

fn bios_video_teletype(emu: &mut Emulator) {
    let color = emu.get_register8(Register8::Bl) & 0x0f;
    let ch = emu.get_register8(Register8::Al) as char;

    let terminal_color = BIOS_TO_TERMINAL[(color & 0x07) as usize];
    let bright = if (color & 0x08) != 0 { 1 } else { 0 };
    let buf = format!("\x1b[{};{}m{}\x1b[0m", bright, terminal_color, ch);
    put_string(&buf);
}

pub fn bios_video(emu: &mut Emulator) {
    let func = emu.get_register8(Register8::Ah);
    match func {
        0x0e => bios_video_teletype(emu),
        _ => println!("not implemented BIOS video function: 0x{:02x}", func),
    }
}
