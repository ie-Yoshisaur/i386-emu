mod emulator;
mod instruction;
mod register;

use emulator::Emulator;
use instruction::init_instructions;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: px86 filename\n");
        return;
    }
    let mut emu = Emulator::create_emu(emulator::MEMORY_SIZE, 0x7c00, 0x7c00);
    emulator::read_program_file(&args[1], &mut emu);
    let mut instructions = vec![None; 256];
    init_instructions(&mut instructions);
    while emu.eip < emulator::MEMORY_SIZE as u32
        && emulator::execute_instruction(&mut emu, &instructions)
    {}
    println!("{}", emu);
}

