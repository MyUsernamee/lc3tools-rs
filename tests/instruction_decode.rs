use lc3tools_rs::{Instruction, Operand, Register};

fn test_decode(bytes: u16) -> Instruction {
    Instruction::from(bytes)
}

#[test]
fn test_decode_add() {
    let reg_instr = test_decode(0b0001001010000100);
    let imm_instr = test_decode(0b0001001010101010);

    assert_eq!(
        reg_instr,
        Instruction::ADD {
            dr: Register::R1,
            sr1: Register::R2,
            op: Operand::Register(Register::R4)
        }
    );
    assert_eq!(
        imm_instr,
        Instruction::ADD {
            dr: Register::R1,
            sr1: Register::R2,
            op: Operand::Immediate(0b01010)
        }
    );
}

#[test]
fn test_decode_and() {
    let reg_instr = test_decode(0b0101001010000100);
    let imm_instr = test_decode(0b0101001010101010);

    assert_eq!(
        reg_instr,
        Instruction::AND {
            dr: Register::R1,
            sr1: Register::R2,
            op: Operand::Register(Register::R4)
        }
    );
    assert_eq!(
        imm_instr,
        Instruction::AND {
            dr: Register::R1,
            sr1: Register::R2,
            op: Operand::Immediate(0b01010)
        }
    );
}

#[test]
fn test_decode_br() {
    let n_instr = test_decode(0b0000100010000100);
    let z_instr = test_decode(0b0000010010000100);
    let p_instr = test_decode(0b0000001010000100);

    assert_eq!(
        n_instr,
        Instruction::BR {
            n: true,
            z: false,
            p: false,
            offset: 0b010000100
        }
    );
    assert_eq!(
        z_instr,
        Instruction::BR {
            n: false,
            z: true,
            p: false,
            offset: 0b010000100
        }
    );
    assert_eq!(
        p_instr,
        Instruction::BR {
            n: false,
            z: false,
            p: true,
            offset: 0b010000100
        }
    );
}

#[test]
fn test_decode_jmp() {
    let instr = test_decode(0b1100000010000000u16);

    assert_eq!(
        instr,
        Instruction::JMP {
            base_r: Register::R2
        }
    );
}

#[test]
fn test_decode_jsr() {
    let jsr_instr = test_decode(0b0100100010010010u16);
    let jsrr_instr = test_decode(0b0100000010000000u16);

    assert_eq!(
        jsr_instr,
        Instruction::JSR {
            op: Operand::Immediate(0b00010010010u16)
        }
    );
    assert_eq!(
        jsrr_instr,
        Instruction::JSR {
            op: Operand::Register(Register::R2)
        }
    );
}

#[test]
fn test_decode_ld() {
    let instr = test_decode(0b0010100010010010u16);

    assert_eq!(
        instr,
        Instruction::LD {
            dr: Register::R4,
            offset: 0b010010010u16
        }
    );
}

#[test]
fn test_decode_ldi() {
    let instr = test_decode(0b1010100010010010u16);

    assert_eq!(
        instr,
        Instruction::LDI {
            dr: Register::R4,
            offset: 0b010010010u16
        }
    );
}

#[test]
fn test_decode_ldr() {
    let instr = test_decode(0b0110100010010010u16);

    assert_eq!(
        instr,
        Instruction::LDR {
            dr: Register::R4,
            base_r: Register::R2,
            offset: 0b010010u16
        }
    );
}

#[test]
fn test_decode_lea() {
    let instr = test_decode(0b1110100010010010u16);

    assert_eq!(
        instr,
        Instruction::LEA {
            dr: Register::R4,
            offset: 0b010010010u16
        }
    );
}

#[test]
fn test_decode_not() {
    let instr = test_decode(0b1001001010111111);

    assert_eq!(
        instr,
        Instruction::NOT {
            dr: Register::R1,
            sr: Register::R2,
        }
    );
}

#[test]
fn test_decode_rti() {
    let instr = test_decode(0b1000000000000000u16);

    assert_eq!(instr, Instruction::RTI);
}

#[test]
fn test_decode_st() {
    let instr = test_decode(0b0011100010010010u16);

    assert_eq!(
        instr,
        Instruction::ST {
            sr: Register::R4,
            offset: 0b010010010u16
        }
    );
}

#[test]
fn test_decode_sti() {
    let instr = test_decode(0b1011100010010010u16);

    assert_eq!(
        instr,
        Instruction::STI {
            sr: Register::R4,
            offset: 0b010010010u16
        }
    );
}

#[test]
fn test_decode_str() {
    let instr = test_decode(0b0111100010010010u16);

    assert_eq!(
        instr,
        Instruction::STR {
            sr: Register::R4,
            base_r: Register::R2,
            offset: 0b010010u16
        }
    );
}

#[test]
fn test_decode_trap() {
    let instr = test_decode(0b1111000010010010u16);

    assert_eq!(instr, Instruction::TRAP(0b10010010))
}
