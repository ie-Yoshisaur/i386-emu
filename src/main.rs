use crate::emulator::Emulator;
use std::env;
use std::process;
mod bios;
mod emulator;
mod io;
mod modrm;

const MEMORY_SIZE: usize = 1024 * 1024;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut quiet = false;

    let args: Vec<String> = args
        .into_iter()
        .filter(|arg| {
            if arg == "-q" {
                quiet = true;
                false
            } else {
                true
            }
        })
        .collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        process::exit(1);
    }
    let filename = &args[1];
    let mut emu = Emulator::new(MEMORY_SIZE, 0x7c00, 0x7c00);
    emu.init_instructions();
    emu.read_binary(filename);
    while (emu.eip as usize) < MEMORY_SIZE {
        let code: u8 = emu.get_code8(0);
        if !quiet {
            println!("EIP = {}, Code = 0x{:02X}\n", emu.eip, code);
        }

        if let Some(instruction) = emu.instructions.get(&code).cloned() {
            emu.execute_instruction(&instruction);
        } else {
            println!("Not Implemented: 0x{:02X}", code);
            break;
        }

        if (emu.eip as usize) == 0 {
            println!("end of program.\n");
            break;
        }
    }
    emu.dump_registers();
}
