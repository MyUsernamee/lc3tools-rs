use lc3tools_rs::{Instruction, LC3Simulator, Operand, Register};
use rand::random;

fn from_2c(bits: u16, value: u16) -> u16 {
    let value_mask = (1 << bits - 1) - 1;
    let sign_bit = value & (1 << bits - 1);
    
    let mut v = (value & value_mask) as u16;
    if sign_bit == 1 {
        v = !v + 1;
    }

    v
}

fn to_2c(bits: u16, value: u16) -> u16 {
    let value_mask = (1 << bits) - 1;
    value & value_mask
}

fn decode(bytes: u16) -> Instruction {
    Instruction::from(bytes)
}

fn cond_code(value: u16) -> u16 {
    match (value as i32 >= 0, value) {
        (false, 0) => {0b010},
        (true, _) => {0b011},
        _ => {0b100}
    }
}

fn make_test_sim(instr: Instruction) -> (LC3Simulator, u16, u16, u16) {
    let mut sim = LC3Simulator::new();
    sim.jump_to(0x3000);
    let r0: u16 = random();
    let r1: u16 = random();
    let r2: u16 = random();
    sim.set_register(Register::R0, r0);
    sim.set_register(Register::R1, r1);
    sim.set_register(Register::R2, r2);
    sim.execute(instr);
    (sim, r0, r1, r2)
}

fn make_imm_value(bits: u16) -> u16 {
    random::<u16>() & ((1u16 << bits) - 1)
}

#[test]
fn test_execute_add() {
    let imm = make_imm_value(5);
    let reg_instr = decode(0b0001000001000010);
    let imm_instr = decode(0b0001000001100000 | imm);

    let (reg_sim, _r0, r1,  r2) = make_test_sim(reg_instr);
    let (imm_sim, _i0, i1, _i2) = make_test_sim(imm_instr);

    assert_eq!(reg_sim.get_program_counter(), 0x3001);
    assert_eq!(imm_sim.get_program_counter(), 0x3001);

    assert_eq!(reg_sim.get_register(0), r1.wrapping_add(r2));
    assert_eq!(imm_sim.get_register(0), i1.wrapping_add(from_2c(5, imm)));

    let r = reg_sim.get_register(0) as i16;
    let i = imm_sim.get_register(0) as i16;
    println!("{i}");

    let rcond = reg_sim.get_condition_code();
    let icond = imm_sim.get_condition_code();

    let ercond = (r < 0, r == 0, r > 0);
    let eicond = (i < 0, i == 0, i > 0);

    assert_eq!(rcond, ercond);
    assert_eq!(icond, eicond);
}

#[test]
fn test_execute_and() {
    let imm = make_imm_value(5);
    let reg_instr = decode(0b0101000001000010);
    let imm_instr = decode(0b0101000001100000 | imm);

    let (reg_sim, _r0, r1,  r2) = make_test_sim(reg_instr);
    let (imm_sim, _i0, i1, _i2) = make_test_sim(imm_instr);

    assert_eq!(reg_sim.get_program_counter(), 0x3001);
    assert_eq!(imm_sim.get_program_counter(), 0x3001);

    assert_eq!(reg_sim.get_register(0), r1 & r2);
    assert_eq!(imm_sim.get_register(0), i1 & from_2c(5, imm));

    let r = reg_sim.get_register(0) as i16;
    let i = imm_sim.get_register(0) as i16;

    let rcond = reg_sim.get_condition_code();
    let icond = imm_sim.get_condition_code();

    let ercond = (r < 0, r == 0, r > 0);
    let eicond = (i < 0, i == 0, i > 0);

    assert_eq!(rcond, ercond);
    assert_eq!(icond, eicond);
}

#[test]
fn test_execute_br() {
    let imm = make_imm_value(9);
    let instr = decode(0b0000100000000000 | imm);
    dbg!(instr, imm);

    let mut sim = LC3Simulator::new();
    sim.jump_to(0x3000);
    sim.set_register(0usize, 0);
    sim.set_register(1usize, random());

    sim.execute(Instruction::ADD{dr:Register::R0, sr1:Register::R0, op: Operand::Register(Register::R1)});
    sim.execute(instr);

    let new_pc = if (sim.get_register(0) as i16) < 0 { 0x3001u16.wrapping_add(from_2c(9, imm)) } else { 0x3001u16 } ;

    assert_eq!(sim.get_program_counter(), new_pc);
}
//
// #[test]
// fn test_execute_jmp() {
//     let instr = decode(0b1100000010000000u16);
//
//     assert_eq!(instr, Instruction::JMP { 
//         base_r: Register::R2
//     });
// }
//
// #[test]
// fn test_execute_jsr() {
//     let jsr_instr  = decode(0b0100100010010010u16);
//     let jsrr_instr = decode(0b0100000010000000u16);
//
//     assert_eq!(jsr_instr, Instruction::JSR { op: Operand::Immediate(0b00010010010u16) });
//     assert_eq!(jsrr_instr, Instruction::JSR { op: Operand::Register(Register::R2) });
// }
//
// #[test]
// fn test_execute_ld() {
//     let instr  = decode(0b0010100010010010u16);
//
//     assert_eq!(instr, Instruction::LD { dr: Register::R4, offset: 0b010010010u16});
// }
//
// #[test]
// fn test_execute_ldi() {
//     let instr  = decode(0b1010100010010010u16);
//
//     assert_eq!(instr, Instruction::LDI { dr: Register::R4, offset: 0b010010010u16});
// }
//
// #[test]
// fn test_execute_ldr() {
//     let instr  = decode(0b0110100010010010u16);
//
//     assert_eq!(instr, Instruction::LDR { dr: Register::R4, base_r: Register::R2, offset: 0b010010u8});
// }
//
// #[test]
// fn test_execute_lea() {
//     let instr  = decode(0b1110100010010010u16);
//
//     assert_eq!(instr, Instruction::LEA { dr: Register::R4, offset: 0b010010010u16});
// }
//
// #[test]
// fn test_execute_not() {
//     let instr = decode(0b1001001010111111);
//
//     assert_eq!(instr, Instruction::NOT { 
//         dr: Register::R1,
//         sr: Register::R2,
//     });
// }
//
// #[test]
// fn test_execute_rti() {
//     let instr  = decode(0b1000000000000000u16);
//
//     assert_eq!(instr, Instruction::RTI);
// }
//
// #[test]
// fn test_execute_st() {
//     let instr  = decode(0b0011100010010010u16);
//
//     assert_eq!(instr, Instruction::ST { sr: Register::R4, offset: 0b010010010u16});
// }
//
// #[test]
// fn test_execute_sti() {
//     let instr  = decode(0b1011100010010010u16);
//
//     assert_eq!(instr, Instruction::STI { sr: Register::R4, offset: 0b010010010u16});
// }
//
// #[test]
// fn test_execute_str() {
//     let instr  = decode(0b0111100010010010u16);
//
//     assert_eq!(instr, Instruction::STR { sr: Register::R4, base_r: Register::R2, offset: 0b010010u8});
// }
//
// #[test]
// fn test_execute_trap() {
//     let instr  = decode(0b1111000010010010u16);
//
//     assert_eq!(instr, Instruction::TRAP(0b10010010) )
// }

