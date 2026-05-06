use super::registers::*;
use super::utils::sign_extend;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operand {
    Immediate(u16),
    Register(Register),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    ADD {
        dr: Register,
        sr1: Register,
        op: Operand,
    },
    AND {
        dr: Register,
        sr1: Register,
        op: Operand,
    },
    BR {
        n: bool,
        z: bool,
        p: bool,
        offset: u16,
    },
    JMP {
        base_r: Register,
    },
    JSR {
        op: Operand,
    },
    LD {
        dr: Register,
        offset: u16,
    },
    LDI {
        dr: Register,
        offset: u16,
    },
    LDR {
        dr: Register,
        base_r: Register,
        offset: u16,
    },
    LEA {
        dr: Register,
        offset: u16,
    },
    NOT {
        dr: Register,
        sr: Register,
    },
    RTI,
    ST {
        sr: Register,
        offset: u16,
    },
    STI {
        sr: Register,
        offset: u16,
    },
    STR {
        sr: Register,
        base_r: Register,
        offset: u16,
    },
    TRAP(u8),
    NOOP,
}

impl From<u16> for Instruction {
    fn from(value: u16) -> Self {
        // Get first 4 bits.
        let memonic = value >> 12;

        match memonic {
            0b0001 => Instruction::ADD {
                dr: Register::from((value >> 9) & 0b111),
                sr1: Register::from((value >> 6) & 0b111),
                op: if (value >> 5) & 0b1 == 1 {
                    Operand::Immediate(sign_extend(5, value))
                } else {
                    Operand::Register(Register::from(value & 0b111))
                },
            },
            0b0101 => Instruction::AND {
                dr: Register::from((value >> 9) & 0b111),
                sr1: Register::from((value >> 6) & 0b111),
                op: if (value >> 5) & 0b1 == 1 {
                    Operand::Immediate(sign_extend(5, value))
                } else {
                    Operand::Register(Register::from(value & 0b111))
                },
            },
            0b0000 => Instruction::BR {
                n: (value >> 11) & 0b1 == 1,
                z: (value >> 10) & 0b1 == 1,
                p: (value >> 9) & 0b1 == 1,
                offset: sign_extend(9, value),
            },
            0b1100 => Instruction::JMP {
                base_r: Register::from((value >> 6) & 0b111),
            },
            0b0100 => Instruction::JSR {
                op: if ((value >> 11) & 0b1) == 1 {
                    Operand::Immediate(sign_extend(10, value))
                } else {
                    Operand::Register(Register::from((value >> 6) & 0b111))
                },
            },
            0b0010 => Instruction::LD {
                dr: Register::from((value >> 9) & 0b111),
                offset: sign_extend(9, value),
            },
            0b1010 => Instruction::LDI {
                dr: Register::from((value >> 9) & 0b111),
                offset: sign_extend(9, value),
            },
            0b0110 => Instruction::LDR {
                dr: Register::from((value >> 9) & 0b111),
                base_r: Register::from((value >> 6) & 0b111),
                offset: sign_extend(6, value),
            },
            0b1110 => Instruction::LEA {
                dr: Register::from((value >> 9) & 0b111),
                offset: sign_extend(9, value),
            },
            0b1001 => Instruction::NOT {
                dr: Register::from((value >> 9) & 0b111),
                sr: Register::from((value >> 6) & 0b111),
            },
            0b1000 => Instruction::RTI,
            0b0011 => Instruction::ST {
                sr: Register::from((value >> 9) & 0b111),
                offset: sign_extend(9, value),
            },
            0b1011 => Instruction::STI {
                sr: Register::from((value >> 9) & 0b111),
                offset: sign_extend(9, value),
            },
            0b0111 => Instruction::STR {
                sr: Register::from((value >> 9) & 0b111),
                base_r: Register::from((value >> 6) & 0b111),
                offset: sign_extend(6, value),
            },
            0b1111 => Instruction::TRAP(value as u8),
            _ => Instruction::NOOP,
        }
    }
}
