
/// Simple enum to represent all general purpose registers.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Register {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
}

impl From<u16> for Register {
    fn from(value: u16) -> Self {
        let lower_bits = value & 0b111;
        match lower_bits {
            0 => Register::R0,
            1 => Register::R1,
            2 => Register::R2,
            3 => Register::R3,
            4 => Register::R4,
            5 => Register::R5,
            6 => Register::R6,
            7 => Register::R7,
            _ => Register::R0,
        }
    }
}

impl Into<usize> for Register {
    fn into(self) -> usize {
        return self as usize;
    }
}
