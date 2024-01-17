use crate::register::Register;
use crate::register::REGISTERS_NAME;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::instruction::{get_code8, InstructionFunc};

pub const MEMORY_SIZE: usize = 1024 * 1024;
const REGISTER_SIZE: usize = 8;

pub struct Emulator {
    pub registers: [u32; REGISTER_SIZE],
    pub eflags: u32,
    pub memory: Vec<u8>,
    pub eip: u32,
}

impl Emulator {
    pub fn create_emu(size: usize, eip: u32, esp: u32) -> Self {
        let mut regs: [u32; REGISTER_SIZE] = [0; REGISTER_SIZE];
        regs[Register::ESP.as_usize()] = esp;
        Self {
            registers: regs,
            eflags: 0,
            memory: vec![0; size],
            eip,
        }
    }
}

impl fmt::Display for Emulator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, &reg) in self.registers.iter().enumerate() {
            writeln!(f, "{} = {:08X}", REGISTERS_NAME[i], reg)?;
        }
        writeln!(f, "EIP = {:08X}", self.eip)
    }
}

pub fn read_program_file(filename: &str, emu: &mut Emulator) {
    let filepath = Path::new(filename);
    let mut file = match File::open(&filepath) {
        Err(e) => panic!("Couldn't open {}: {}", filepath.display(), e),
        Ok(file) => file,
    };
    file.read(&mut emu.memory[0x7c00..0x7c00 + 0x200]).unwrap();
}

pub fn execute_instruction(emu: &mut Emulator, instructions: &[Option<InstructionFunc>]) -> bool {
    let code = get_code8(emu, 0);
    println!("EIP = {:08X}, Code = {:02X}", emu.eip, code);
    match &instructions[code as usize] {
        Some(f) => {
            f(emu);
            if emu.eip == 0x00 {
                println!("\n\nend of program.\n\n");
                return false;
            }
        }
        None => {
            println!("\n\nNot Implemented: {:X}", code);
            return false;
        }
    }
    return true;
}
