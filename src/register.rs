pub enum Register {
    EAX,
    ECX,
    EDX,
    EBX,
    ESP,
    EBP,
    ESI,
    EDI,
}

impl Register {
    pub fn as_usize(&self) -> usize {
        match self {
            Self::EAX => 0,
            Self::ECX => 1,
            Self::EDX => 2,
            Self::EBX => 3,
            Self::ESP => 4,
            Self::EBP => 5,
            Self::ESI => 6,
            Self::EDI => 7,
        }
    }
}

const REGISTER_SIZE: usize = 8;

pub const REGISTERS_NAME: [&str; REGISTER_SIZE] =
    ["EAX", "ECX", "EDX", "EBX", "ESP", "EBP", "ESI", "EDI"];
