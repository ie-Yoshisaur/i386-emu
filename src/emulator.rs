use crate::bios::bios_video;
use crate::io::io_in8;
use crate::io::io_out8;
use crate::modrm::ModRM;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

const CARRY_FLAG: u32 = 1 << 0;
const ZERO_FLAG: u32 = 1 << 1;
const SIGN_FLAG: u32 = 1 << 2;
const OVERFLOW_FLAG: u32 = 1 << 3;

enum Register {
    Eax,
    Ecx,
    Edx,
    Ebx,
    Esp,
    Ebp,
    Esi,
    Edi,
}

pub enum Register8 {
    Al,
    Cl,
    Dl,
    Bl,
    Ah,
    Ch,
    Dh,
    Bh,
}

impl Register8 {
    pub fn from_usize(value: usize) -> Option<Self> {
        match value {
            0 => Some(Register8::Al),
            1 => Some(Register8::Cl),
            2 => Some(Register8::Dl),
            3 => Some(Register8::Bl),
            4 => Some(Register8::Ah),
            5 => Some(Register8::Ch),
            6 => Some(Register8::Dh),
            7 => Some(Register8::Bh),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum Instruction {
    MovR8Imm8,
    MovR32Imm32,
    MovR8Rm8,
    MovR32Rm32,
    AddRm32R32,
    MovRm8R8,
    MovRm32R32,
    IncR32,
    PushR32,
    PopR32,
    PushImm32,
    PushImm8,
    Code83,
    MovRm32Imm32,
    InAlDx,
    OutDxAl,
    CodeFf,
    CallRel32,
    Ret,
    Leave,
    ShortJump,
    NearJump,
    CmpAlImm8,
    CmpEaxImm32,
    CmpR32Rm32,
    Jc,
    Jnc,
    Jz,
    Jnz,
    Js,
    Jns,
    Jo,
    Jno,
    Jl,
    Jle,
    Swi,
}

pub struct Emulator {
    registers: [u32; 8],
    eflags: u32,
    memory: Vec<u8>,
    pub eip: u32,
    pub instructions: HashMap<u8, Instruction>,
}

impl Emulator {
    pub fn new(memory_size: usize, eip: u32, esp: u32) -> Self {
        let mut emu = Emulator {
            registers: [0; 8],
            eflags: 0,
            memory: vec![0; memory_size],
            eip,
            instructions: HashMap::new(),
        };
        emu.registers[Register::Esp as usize] = esp;
        emu
    }

    pub fn get_code8(&self, index: usize) -> u8 {
        self.memory[self.eip as usize + index]
    }

    pub fn get_sign_code8(&self, index: usize) -> i8 {
        self.memory[self.eip as usize + index] as i8
    }

    fn get_code32(&self, index: usize) -> u32 {
        let mut ret = 0;
        for i in 0..4 {
            ret |= (self.get_code8(index + i) as u32) << (i * 8);
        }
        ret
    }

    pub fn get_sign_code32(&self, index: usize) -> i32 {
        self.get_code32(index) as i32
    }

    pub fn get_register8(&self, reg: Register8) -> u8 {
        match reg {
            Register8::Al => self.registers[Register::Eax as usize] as u8,
            Register8::Cl => self.registers[Register::Ecx as usize] as u8,
            Register8::Dl => self.registers[Register::Edx as usize] as u8,
            Register8::Bl => self.registers[Register::Ebx as usize] as u8,
            Register8::Ah => (self.registers[Register::Eax as usize] >> 8) as u8,
            Register8::Ch => (self.registers[Register::Ecx as usize] >> 8) as u8,
            Register8::Dh => (self.registers[Register::Edx as usize] >> 8) as u8,
            Register8::Bh => (self.registers[Register::Ebx as usize] >> 8) as u8,
        }
    }

    pub fn get_register32(&self, index: usize) -> u32 {
        self.registers[index]
    }

    pub fn set_register8(&mut self, reg: Register8, value: u8) {
        match reg {
            Register8::Al | Register8::Cl | Register8::Dl | Register8::Bl => {
                let index = reg as usize;
                let r = self.registers[index] & 0xffffff00;
                self.registers[index] = r | u32::from(value);
            }
            Register8::Ah | Register8::Ch | Register8::Dh | Register8::Bh => {
                let index = (reg as usize) - 4;
                let r = self.registers[index] & 0xffff00ff;
                self.registers[index] = r | ((u32::from(value)) << 8);
            }
        }
    }

    pub fn set_register32(&mut self, index: usize, value: u32) {
        self.registers[index] = value;
    }

    pub fn set_memory8(&mut self, address: u32, value: u8) {
        self.memory[address as usize] = value;
    }

    pub fn set_memory32(&mut self, address: u32, value: u32) {
        for i in 0..4 {
            self.set_memory8(address + i, ((value >> (i * 8)) & 0xff) as u8);
        }
    }

    pub fn get_memory8(&self, address: u32) -> u8 {
        self.memory[address as usize]
    }

    pub fn get_memory32(&self, address: u32) -> u32 {
        let mut ret = 0;
        for i in 0..4 {
            ret |= (self.get_memory8(address + i) as u32) << (8 * i);
        }
        ret
    }

    fn push32(&mut self, value: u32) {
        let address = self.get_register32(4) - 4;
        self.set_register32(4, address);
        self.set_memory32(address, value);
    }

    fn pop32(&mut self) -> u32 {
        let address = self.get_register32(4);
        let ret = self.get_memory32(address);
        self.set_register32(4, address + 4);
        ret
    }

    fn set_carry(&mut self, is_carry: bool) {
        if is_carry {
            self.eflags |= CARRY_FLAG;
        } else {
            self.eflags &= !CARRY_FLAG;
        }
    }

    fn set_zero(&mut self, is_zero: bool) {
        if is_zero {
            self.eflags |= ZERO_FLAG;
        } else {
            self.eflags &= !ZERO_FLAG;
        }
    }

    fn set_sign(&mut self, is_sign: bool) {
        if is_sign {
            self.eflags |= SIGN_FLAG;
        } else {
            self.eflags &= !SIGN_FLAG;
        }
    }

    fn set_overflow(&mut self, is_overflow: bool) {
        if is_overflow {
            self.eflags |= OVERFLOW_FLAG;
        } else {
            self.eflags &= !OVERFLOW_FLAG;
        }
    }

    fn is_carry(&self) -> bool {
        self.eflags & CARRY_FLAG != 0
    }

    fn is_zero(&self) -> bool {
        self.eflags & ZERO_FLAG != 0
    }

    fn is_sign(&self) -> bool {
        self.eflags & SIGN_FLAG != 0
    }

    fn is_overflow(&self) -> bool {
        self.eflags & OVERFLOW_FLAG != 0
    }

    fn update_eflags_sub(&mut self, v1: u32, v2: u32, result: u64) {
        let sign1 = v1 >> 31;
        let sign2 = v2 >> 31;
        let signr = (result >> 31) & 1;

        self.set_carry((result >> 32) != 0);
        self.set_zero((result & 0xFFFFFFFF) == 0);
        self.set_sign(signr != 0);
        self.set_overflow((sign1 != sign2) && (u64::from(sign1) != signr));
    }

    pub fn read_binary(&mut self, filename: &str) {
        let mut file = File::open(filename).expect(&format!("Failed to open file: {}", filename));
        file.read(&mut self.memory[0x7c00..0x7c00 + 0x200])
            .expect("Failed to read file into memory");
    }

    pub fn dump_registers(&self) {
        let register_names = ["EAX", "ECX", "EDX", "EBX", "ESP", "EBP", "ESI", "EDI"];
        for (i, &name) in register_names.iter().enumerate() {
            println!("{} = {:08X}", name, self.registers[i]);
        }
        println!("EIP = {:08X}", self.eip);
    }

    fn parse_modrm(&mut self) -> ModRM {
        let code = self.get_code8(0);
        let mod_val = (code & 0xC0) >> 6;
        let opecode = (code & 0x38) >> 3;
        let rm = code & 0x07;

        self.eip += 1;

        let mut modrm = ModRM {
            mod_val,
            opecode,
            rm,
            sib: None,
            disp8: None,
            disp32: None,
        };

        if modrm.mod_val != 3 && modrm.rm == 4 {
            modrm.sib = Some(self.get_code8(0));
            self.eip += 1;
        }

        match modrm.mod_val {
            0 => {
                if modrm.rm == 5 {
                    modrm.disp32 = Some(self.get_sign_code32(0));
                    self.eip += 4;
                }
            }
            1 => {
                modrm.disp8 = Some(self.get_sign_code8(0));
                self.eip += 1;
            }
            2 => {
                modrm.disp32 = Some(self.get_sign_code32(0));
                self.eip += 4;
            }
            _ => {}
        }

        modrm
    }

    pub fn execute_instruction(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::MovR8Imm8 => {
                self.mov_r8_imm8();
            }
            Instruction::MovR32Imm32 => {
                self.mov_r32_imm32();
            }
            Instruction::MovR8Rm8 => {
                self.mov_r8_rm8();
            }
            Instruction::MovR32Rm32 => {
                self.mov_r32_rm32();
            }
            Instruction::AddRm32R32 => {
                self.add_rm32_r32();
            }
            Instruction::MovRm8R8 => {
                self.mov_rm8_r8();
            }
            Instruction::MovRm32R32 => {
                self.mov_rm32_r32();
            }
            Instruction::IncR32 => {
                self.inc_r32();
            }
            Instruction::PushR32 => {
                self.push_r32();
            }
            Instruction::PopR32 => {
                self.pop_r32();
            }
            Instruction::PushImm32 => {
                self.push_imm32();
            }
            Instruction::PushImm8 => {
                self.push_imm8();
            }
            Instruction::Code83 => {
                self.code_83();
            }
            Instruction::MovRm32Imm32 => {
                self.mov_rm32_imm32();
            }
            Instruction::InAlDx => {
                self.in_al_dx();
            }
            Instruction::OutDxAl => {
                self.out_dx_al();
            }
            Instruction::CodeFf => {
                self.code_ff();
            }
            Instruction::CallRel32 => {
                self.call_rel32();
            }
            Instruction::Ret => {
                self.ret();
            }
            Instruction::Leave => {
                self.leave();
            }
            Instruction::ShortJump => {
                self.short_jump();
            }
            Instruction::NearJump => {
                self.near_jump();
            }
            Instruction::CmpAlImm8 => {
                self.cmp_al_imm8();
            }
            Instruction::CmpEaxImm32 => {
                self.cmp_eax_imm32();
            }
            Instruction::CmpR32Rm32 => {
                self.cmp_r32_rm32();
            }
            Instruction::Jc => {
                self.jc();
            }
            Instruction::Jnc => {
                self.jnc();
            }
            Instruction::Jz => {
                self.jz();
            }
            Instruction::Jnz => {
                self.jnz();
            }
            Instruction::Js => {
                self.js();
            }
            Instruction::Jns => {
                self.jns();
            }
            Instruction::Jo => {
                self.jo();
            }
            Instruction::Jno => {
                self.jno();
            }
            Instruction::Jl => {
                self.jl();
            }
            Instruction::Jle => {
                self.jle();
            }
            Instruction::Swi => {
                self.swi();
            }
        }
    }

    fn mov_r8_imm8(&mut self) {
        if let Some(reg) = Register8::from_usize((self.get_code8(0) - 0xB0) as usize) {
            let value = self.get_code8(1);
            self.set_register8(reg, value);
            self.eip += 2;
        } else {
            eprintln!("Error: Invalid register index in mov_r8_imm8");
        }
    }

    fn mov_r32_imm32(&mut self) {
        let reg = (self.get_code8(0) - 0xB8) as usize;
        let value = self.get_code32(1);
        self.set_register32(reg, value);
        self.eip += 5;
    }

    fn mov_r8_rm8(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        let rm8 = modrm.get_rm8(self);
        modrm.set_r8(self, rm8);
    }

    fn mov_r32_rm32(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        let rm32 = modrm.get_rm32(self);
        modrm.set_r32(self, rm32);
    }

    fn add_rm32_r32(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        let r32 = modrm.get_r32(self);
        let rm32 = modrm.get_rm32(self);
        modrm.set_rm32(self, rm32 + r32);
    }

    fn mov_rm8_r8(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        let r8 = modrm.get_r8(self);
        modrm.set_rm8(self, r8);
    }

    fn mov_rm32_r32(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        let r32 = modrm.get_r32(&self);
        modrm.set_rm32(self, r32);
    }

    fn inc_r32(&mut self) {
        let reg = (self.get_code8(0) - 0x40) as usize;
        let value = self.get_register32(reg) + 1;
        self.set_register32(reg, value);
        self.eip += 1;
    }

    fn push_r32(&mut self) {
        let reg = (self.get_code8(0) - 0x50) as usize;
        let value = self.get_register32(reg);
        self.push32(value);
        self.eip += 1;
    }

    fn pop_r32(&mut self) {
        let reg = (self.get_code8(0) - 0x58) as usize;
        let value = self.pop32();
        self.set_register32(reg, value);
        self.eip += 1;
    }

    fn push_imm32(&mut self) {
        let value = self.get_code32(1);
        self.push32(value);
        self.eip += 5;
    }

    fn push_imm8(&mut self) {
        let value = self.get_code8(1) as u32;
        self.push32(value);
        self.eip += 2;
    }

    fn add_rm32_imm8(&mut self, modrm: &ModRM) {
        let rm32 = modrm.get_rm32(self);
        let imm8 = self.get_sign_code8(0) as i32 as u32;
        self.eip += 1;
        modrm.set_rm32(self, rm32.wrapping_add(imm8));
    }

    fn cmp_rm32_imm8(&mut self, modrm: &ModRM) {
        let rm32 = modrm.get_rm32(self);
        let imm8 = self.get_sign_code8(0) as i32 as u32;
        self.eip += 1;
        let result = (rm32 as u64).wrapping_sub(imm8 as u64);
        self.update_eflags_sub(rm32, imm8, result);
    }

    fn sub_rm32_imm8(&mut self, modrm: &ModRM) {
        let rm32 = modrm.get_rm32(self);
        let imm8 = self.get_sign_code8(0) as i32 as u32;
        self.eip += 1;
        let result = (rm32 as u64).wrapping_sub(imm8 as u64);
        modrm.set_rm32(self, rm32.wrapping_sub(imm8));
        self.update_eflags_sub(rm32, imm8, result);
    }

    fn code_83(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        match modrm.opecode {
            0 => self.add_rm32_imm8(&modrm),
            5 => self.sub_rm32_imm8(&modrm),
            7 => self.cmp_rm32_imm8(&modrm),
            _ => {
                println!("not implemented: 83 /{}", modrm.opecode);
                std::process::exit(1);
            }
        }
    }

    fn mov_rm32_imm32(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        let value = self.get_code32(0);
        self.eip += 4;
        modrm.set_rm32(self, value);
    }

    fn in_al_dx(&mut self) {
        let address = self.get_register32(Register::Edx as usize) & 0xffff;
        let value = io_in8(address as u16);
        self.set_register8(Register8::Al, value);
        self.eip += 1;
    }

    fn out_dx_al(&mut self) {
        let address = self.get_register32(Register::Edx as usize) & 0xffff;
        let value = self.get_register8(Register8::Al);
        io_out8(address as u16, value);
        self.eip += 1;
    }

    fn inc_rm32(&mut self, modrm: &ModRM) {
        let value = modrm.get_rm32(self);
        modrm.set_rm32(self, value.wrapping_add(1));
    }

    fn code_ff(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        match modrm.opecode {
            0 => self.inc_rm32(&modrm),
            _ => {
                println!("not implemented: FF /{}", modrm.opecode);
                std::process::exit(1);
            }
        }
    }

    fn call_rel32(&mut self) {
        let diff = self.get_sign_code32(1) as i32;
        self.push32(self.eip + 5);
        self.eip = self.eip.wrapping_add(diff as u32 + 5);
    }

    fn ret(&mut self) {
        self.eip = self.pop32();
    }

    fn leave(&mut self) {
        let ebp = self.get_register32(Register::Ebp as usize);
        let popped_value = self.pop32();
        self.set_register32(Register::Esp as usize, ebp);
        self.set_register32(Register::Ebp as usize, popped_value);
        self.eip += 1;
    }

    fn short_jump(&mut self) {
        let diff = self.get_sign_code8(1) as i8;
        self.eip = self.eip.wrapping_add(diff as u32 + 2);
    }

    fn near_jump(&mut self) {
        let diff = self.get_sign_code32(1) as i32;
        self.eip = self.eip.wrapping_add(diff as u32 + 5);
    }

    fn cmp_al_imm8(&mut self) {
        let value = self.get_code8(1);
        let al = self.get_register8(Register8::Al);
        let result = (al as u64).wrapping_sub(value as u64);
        self.update_eflags_sub(al as u32, value as u32, result);
        self.eip += 2;
    }

    fn cmp_eax_imm32(&mut self) {
        let value = self.get_code32(1);
        let eax = self.get_register32(Register::Eax as usize);
        let result = (eax as u64).wrapping_sub(value as u64);
        self.update_eflags_sub(eax, value, result);
        self.eip += 5;
    }

    fn cmp_r32_rm32(&mut self) {
        self.eip += 1;
        let modrm = self.parse_modrm();
        let r32 = modrm.get_r32(self);
        let rm32 = modrm.get_rm32(self);
        let result = (r32 as u64).wrapping_sub(rm32 as u64);
        self.update_eflags_sub(r32, rm32, result);
    }

    fn conditional_jump(&mut self, condition: bool) {
        let diff = if condition {
            self.get_sign_code8(1) as i8
        } else {
            0
        };
        self.eip = self.eip.wrapping_add(diff as u32 + 2);
    }

    fn jc(&mut self) {
        self.conditional_jump(self.is_carry());
    }

    fn jnc(&mut self) {
        self.conditional_jump(!self.is_carry());
    }

    fn jz(&mut self) {
        self.conditional_jump(self.is_zero());
    }

    fn jnz(&mut self) {
        self.conditional_jump(!self.is_zero());
    }

    fn js(&mut self) {
        self.conditional_jump(self.is_sign());
    }

    fn jns(&mut self) {
        self.conditional_jump(!self.is_sign());
    }

    fn jo(&mut self) {
        self.conditional_jump(self.is_overflow());
    }

    fn jno(&mut self) {
        self.conditional_jump(!self.is_overflow());
    }

    fn jl(&mut self) {
        let diff = if self.is_sign() != self.is_overflow() {
            self.get_sign_code8(1) as i8
        } else {
            0
        };
        self.eip = self.eip.wrapping_add(diff as u32 + 2);
    }

    fn jle(&mut self) {
        let diff = if self.is_zero() || (self.is_sign() != self.is_overflow()) {
            self.get_sign_code8(1) as i8
        } else {
            0
        };
        self.eip = self.eip.wrapping_add(diff as u32 + 2);
    }

    fn swi(&mut self) {
        let int_index = self.get_code8(1);
        self.eip += 2;

        match int_index {
            0x10 => bios_video(self),
            _ => println!("unknown interrupt: 0x{:02x}", int_index),
        }
    }

    pub fn init_instructions(&mut self) {
        self.instructions.insert(0x01, Instruction::AddRm32R32);

        self.instructions.insert(0x3B, Instruction::CmpR32Rm32);
        self.instructions.insert(0x3C, Instruction::CmpAlImm8);
        self.instructions.insert(0x3D, Instruction::CmpEaxImm32);

        for i in 0x40..0x48 {
            self.instructions.insert(i, Instruction::IncR32);
        }

        for i in 0x50..0x58 {
            self.instructions.insert(i, Instruction::PushR32);
        }

        for i in 0x58..0x60 {
            self.instructions.insert(i, Instruction::PopR32);
        }

        self.instructions.insert(0x68, Instruction::PushImm32);
        self.instructions.insert(0x6A, Instruction::PushImm8);

        self.instructions.insert(0x70, Instruction::Jo);
        self.instructions.insert(0x71, Instruction::Jno);
        self.instructions.insert(0x72, Instruction::Jc);
        self.instructions.insert(0x73, Instruction::Jnc);
        self.instructions.insert(0x74, Instruction::Jz);
        self.instructions.insert(0x75, Instruction::Jnz);
        self.instructions.insert(0x78, Instruction::Js);
        self.instructions.insert(0x79, Instruction::Jns);
        self.instructions.insert(0x7C, Instruction::Jl);
        self.instructions.insert(0x7E, Instruction::Jle);

        self.instructions.insert(0x83, Instruction::Code83);
        self.instructions.insert(0x88, Instruction::MovRm8R8);
        self.instructions.insert(0x89, Instruction::MovRm32R32);
        self.instructions.insert(0x8A, Instruction::MovR8Rm8);
        self.instructions.insert(0x8B, Instruction::MovR32Rm32);

        for i in 0xB0..0xB8 {
            self.instructions.insert(i, Instruction::MovR8Imm8);
        }
        for i in 0xB8..0xC0 {
            self.instructions.insert(i, Instruction::MovR32Imm32);
        }

        self.instructions.insert(0xC3, Instruction::Ret);
        self.instructions.insert(0xC7, Instruction::MovRm32Imm32);
        self.instructions.insert(0xC9, Instruction::Leave);

        self.instructions.insert(0xCD, Instruction::Swi);

        self.instructions.insert(0xE8, Instruction::CallRel32);
        self.instructions.insert(0xE9, Instruction::NearJump);
        self.instructions.insert(0xEB, Instruction::ShortJump);
        self.instructions.insert(0xEC, Instruction::InAlDx);
        self.instructions.insert(0xEE, Instruction::OutDxAl);
        self.instructions.insert(0xFF, Instruction::CodeFf);
    }
}
