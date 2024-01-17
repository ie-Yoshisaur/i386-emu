pub enum RegOpcode {
    Opecode(u8),
    RegIndex(u8),
}

pub enum Disp {
    Disp8(i8),
    Disp32(u32),
}

pub struct ModRM {
    pub mod_: u8,
    pub reg_opcode: RegOpcode,
    pub rm: u8,
    pub sib: u8,
    pub disp: Disp,
}
