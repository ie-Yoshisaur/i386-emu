use crate::emulator::Emulator;

pub type InstructionFunc = fn(&mut Emulator);

pub fn get_code8(emu: &Emulator, index: i32) -> u32 {
    emu.memory[emu.eip as usize + index as usize] as u32
}

pub fn get_sign_code8(emu: &Emulator, index: i32) -> i32 {
    emu.memory[emu.eip as usize + index as usize] as i32
}

pub fn get_code32(emu: &Emulator, index: i32) -> u32 {
    let mut ret = 0;
    for i in 0..4 {
        ret |= get_code8(emu, index + i) << (i * 8);
    }
    ret
}

/// Emulates the "mov" instruction for a 32-bit register and a 32-bit immediate value.
///
/// # Opcode mapping
///
/// | Opcode | Instruction        |
/// |--------|--------------------|
/// | 0xB8   | mov eax, <value>  |
/// | 0xB9   | mov ecx, <value>  |
/// | 0xBA   | mov edx, <value>  |
/// | 0xBB   | mov ebx, <value>  |
/// | 0xBC   | mov esp, <value>  |
/// | 0xBD   | mov ebp, <value>  |
/// | 0xBE   | mov esi, <value>  |
/// | 0xBF   | mov edi, <value>  |
///
/// # Parameters
///
/// * `emu` - Mutable reference to the CPU emulator instance.
pub fn mov_r32_imm32(emu: &mut Emulator) {
    // Gather the opcode, subtract 0xB8 to find the specific register index,
    // since the opcodes from 0xB8 to 0xBF correspond to the registers eax, ecx, edx, ebx, esp, ebp, esi, edi respectively.
    // For instance, if opcode is 0xB9 (mov ecx), then the index will be 0xB9 - 0xB8 = 1.
    let reg: u8 = (get_code8(emu, 0) - 0xB8) as u8;

    let value: u32 = get_code32(emu, 1); // Extracts the 32-bit immediate value from the code
    emu.registers[reg as usize] = value as u32; // Assigns the immediate value to the respective register
    emu.eip += 5; // Advances the instruction pointer by 5 (1 byte opcode + 4 bytes for 32-bit immediate value).
}

pub fn short_jump(emu: &mut Emulator) {
    let diff: i8 = get_sign_code8(emu, 1) as i8;
    emu.eip = (emu.eip as i32 + diff as i32 + 2) as u32;
}

pub fn near_jump(emu: &mut Emulator) {
    let diff: i32 = get_code32(emu, 1) as i32;
    emu.eip = (emu.eip as i32 + diff + 5) as u32;
}

pub fn init_instructions(instructions: &mut Vec<Option<InstructionFunc>>) {
    for i in 0..8 {
        instructions[0xB8 + i] = Some(mov_r32_imm32);
    }
    instructions[0xE9] = Some(near_jump);
    instructions[0xEB] = Some(short_jump);
}
