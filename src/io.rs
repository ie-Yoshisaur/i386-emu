use std::io::{self, Read, Write};

pub fn io_in8(address: u16) -> u8 {
    match address {
        0x03f8 => {
            let input_byte = io::stdin().bytes().next().unwrap_or(Ok(0));
            match input_byte {
                Ok(b) => b,
                Err(_) => 0,
            }
        }
        _ => 0,
    }
}

pub fn io_out8(address: u16, value: u8) {
    match address {
        0x03f8 => {
            io::stdout().write_all(&[value]).unwrap();
            io::stdout().flush().unwrap();
        }
        _ => (),
    }
}
