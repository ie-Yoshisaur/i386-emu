use crate::emulator::Emulator;
use crate::emulator::Register8;

pub struct ModRM {
    pub mod_val: u8,
    pub opecode: u8,
    pub rm: u8,
    pub sib: Option<u8>,
    pub disp8: Option<i8>,
    pub disp32: Option<i32>,
}

impl ModRM {
    fn new() -> Self {
        Self {
            mod_val: 0,
            opecode: 0,
            rm: 0,
            sib: None,
            disp8: None,
            disp32: None,
        }
    }

    fn calc_memory_address(&self, emu: &Emulator) -> u32 {
        match self.mod_val {
            0 => match self.rm {
                4 => panic!("not implemented ModRM mod = 0, rm = 4"),
                5 => self.disp32.expect("disp32 is None") as u32,
                _ => emu.get_register32(self.rm as usize),
            },
            1 => {
                if self.rm == 4 {
                    panic!("not implemented ModRM mod = 1, rm = 4");
                }
                let disp8_to_u32 = self.disp8.expect("disp8 is None") as i32 as u32;
                emu.get_register32(self.rm as usize)
                    .wrapping_add(disp8_to_u32)
            }
            2 => {
                if self.rm == 4 {
                    panic!("not implemented ModRM mod = 2, rm = 4");
                }
                let disp32_to_u32 = self.disp32.expect("disp32 is None") as u32;
                emu.get_register32(self.rm as usize)
                    .wrapping_add(disp32_to_u32)
            }
            3 => panic!("not implemented ModRM mod = 3"),
            _ => unreachable!(),
        }
    }

    pub fn set_rm8(&self, emu: &mut Emulator, value: u8) {
        if self.mod_val == 3 {
            if let Some(reg) = Register8::from_usize(self.rm as usize) {
                emu.set_register8(reg, value);
            } else {
                panic!("Invalid register index: {}", self.rm);
            }
        } else {
            let address = self.calc_memory_address(emu);
            emu.set_memory8(address, value);
        }
    }

    pub fn set_rm32(&self, emu: &mut Emulator, value: u32) {
        if self.mod_val == 3 {
            emu.set_register32(self.rm as usize, value);
        } else {
            let address = self.calc_memory_address(emu);
            emu.set_memory32(address, value);
        }
    }

    pub fn get_rm8(&self, emu: &Emulator) -> u8 {
        if self.mod_val == 3 {
            if let Some(reg) = Register8::from_usize(self.rm as usize) {
                emu.get_register8(reg)
            } else {
                panic!("Invalid register index: {}", self.rm);
            }
        } else {
            let address = self.calc_memory_address(emu);
            emu.get_memory8(address)
        }
    }

    pub fn get_rm32(&self, emu: &Emulator) -> u32 {
        if self.mod_val == 3 {
            emu.get_register32(self.rm as usize)
        } else {
            let address = self.calc_memory_address(emu);
            emu.get_memory32(address)
        }
    }

    pub fn set_r8(&self, emu: &mut Emulator, value: u8) {
        if let Some(reg) = Register8::from_usize(self.opecode as usize) {
            emu.set_register8(reg, value);
        } else {
            panic!("Invalid opcode for Register8: {}", self.opecode);
        }
    }

    pub fn set_r32(&self, emu: &mut Emulator, value: u32) {
        emu.set_register32(self.opecode as usize, value);
    }

    pub fn get_r8(&self, emu: &Emulator) -> u8 {
        if let Some(reg) = Register8::from_usize(self.opecode as usize) {
            emu.get_register8(reg)
        } else {
            panic!("Invalid opcode for Register8: {}", self.opecode);
        }
    }

    pub fn get_r32(&self, emu: &Emulator) -> u32 {
        emu.get_register32(self.opecode as usize)
    }
}
