use std::{iter::zip, ops::Range, slice::Iter, str::Bytes};

// std::string lc3::utils::getMagicHeader(void) { return "\x1c\x30\x15\xc0\x01"; }
// std::string lc3::utils::getVersionString(void) { return "\x01\x01"; }

const MAGIC_HEADER: &[u8; 5] = b"\x1c\x30\x15\xc0\x01";
const VERSION_STRING: &[u8; 2] = b"\x01\x01";

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

impl From<i16> for Register {
    fn from(value: i16) -> Self {
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
            _ => Register::R0
        }
    }
}

pub enum Operand {
    Immediate(i16),
    Register(Register)
}

pub enum Instruction {
    ADD {dr: Register, sr1: Register, op: Operand},
    AND {dr: Register, sr1: Register, op: Operand},
    BR { n: bool, z: bool, p: bool, offset: i16 },
    JMP {base_r: Register},
    JSR {op: Operand},
    LD {dr: Register, offset: i16},
    LDI {dr: Register, offset: i16},
    LDR {dr: Register, base_r: Register, offset: i8},
    LEA {dr: Register, offset: i16},
    NOT {dr: Register, sr: Register},
    RET,
    RTI,
    ST {sr: Register, offset: i16},
    STI {sr: Register, offset: i16},
    STR {sr: Register, base_r: Register, offset: i8},
    TRAP,
    NOOP,
}

impl From<i16> for Instruction {
    fn from(value: i16) -> Self {
        // Get first 4 bits.
        let memonic = value >> 12;

        match memonic {
            0b0001 => Instruction::ADD {
                dr: Register::from(value >> 8 & 0b111),
                sr1: Register::from(value >> 5 & 0b111),
                op: if value >> 4 & 0b1 == 1 { Operand::Immediate(value & 0b11111) } else {Operand::Register(Register::from(value & 0b111))}
            },
            0b0101 => Instruction::AND {
                dr: Register::from(value >> 8 & 0b111),
                sr1: Register::from(value >> 5 & 0b111),
                op: if value >> 4 & 0b1 == 1 { Operand::Immediate(value & 0b11111) } else {Operand::Register(Register::from(value & 0b111))}
            },
            0b0000 => Instruction::BR { 
                n: value >> 10 & 0b1 == 1, 
                z: value >> 9 & 0b1 == 1, 
                p: value >> 8 & 0b1 == 1, 
                offset: value & 1<<10 - 1 
            },
            0b1100 => Instruction::JMP { base_r: Register::from(value >> 5 & 0b111) }, 
            0b0100 => Instruction::JSR { op: if (value >> 10 & 0b1) == 1 { Operand::Immediate(value & (1 << 12 - 1)) } else {Operand::Register(Register::from(value >> 5 & 0b111))} },
            0b0010 => Instruction::LD { dr: Register::from(value >> 8 & 0b111), offset: value & ((1 << 10) - 1) },
            _ => Instruction::NOOP
        }
    }
}

pub struct LC3Simulator {
    registers: [i16; 8],
    program_counter: i16,
    state: i16,
    memory: Box<[i16; 0xFFFF]>,
    annotations: Box<[Option<String>; 0xFFFF]>,
}

#[derive(Debug)]
pub enum ObjLoadErr {
    MissingData,
    MagicHeader {expected: [u8; 5], got: [u8; 5]},
    VersionString {expected: [u8; 2], got: [u8; 2]}
}

impl LC3Simulator {
    pub fn new() -> LC3Simulator {
        LC3Simulator { registers: [0; 8], program_counter: 0, state: 0, memory: Box::new([0; 0xFFFF]), annotations: Box::new([const { None }; 0xFFFF])}
    }

    pub fn load_obj(&mut self, data: Vec<u8>, jump: bool) -> Result<(), ObjLoadErr> {
        if data.len() < 7 {
            return Err(ObjLoadErr::MissingData);
        }
        let mut ptr = 0; 

        for byte in MAGIC_HEADER {
            if *byte != data[ptr] {
                return Err(ObjLoadErr::MagicHeader { expected: *MAGIC_HEADER, got: data[0..5].try_into().unwrap() })
            }
            ptr += 1
        }
        for byte in VERSION_STRING {
            if *byte != data[ptr] {
                return Err(ObjLoadErr::VersionString { expected: *VERSION_STRING, got: data[5..7].try_into().unwrap() })
            }
            ptr += 1
        }
        
        let mut orig = 0;
        let mut words: Vec<i16> = Vec::new();
        let mut annotations: Vec<Option<String>> = Vec::new();
        while ptr < data.len() {
            let word = i16::from_ne_bytes(data[ptr..ptr+2].try_into().unwrap());
            ptr += 2;

            let is_orig = data[ptr] == 1;
            ptr += 1; 

            let annotation_size = u32::from_ne_bytes(data[ptr..ptr+4].try_into().unwrap());
            ptr += 4;

            let mut annotation = None;
            if annotation_size > 0 {
                annotation = Some(String::from_utf8(data[(ptr as usize)..((ptr as u32 + annotation_size) as usize)].to_vec()).unwrap()) ;
            }

            ptr += annotation_size as usize;
            
            if is_orig {
                orig = word;
            }

            words.push(word);
            annotations.push(annotation);
        }

        for (idx, (word, annotation)) in zip(words, annotations).enumerate() {
            let ptr = idx + orig as usize;
            
            self.memory[ptr] = word;
            self.annotations[ptr] = annotation;
        }

        if jump {
            self.jump_to(orig);
        }

        Ok(())
    }

    pub fn step(&mut self, steps: usize) -> usize {
        for step in 0..steps {
            let memory = self.fetch();
            let instr = LC3Simulator::decode(memory);
            let halt = self.execute(instr);

            if halt {
                return step;
            }
        }

        steps
    }

    pub fn run(&mut self) -> usize {
        let mut steps = 0;

        while {
            let memory = self.fetch();
            let instr = LC3Simulator::decode(memory);
            self.execute(instr)
        } { steps += 1; }

        steps
    }

    pub fn run_to(&mut self, location: i16) -> usize {
        let mut steps = 0;

        while {
            let memory = self.fetch();
            let instr = LC3Simulator::decode(memory);
            self.execute(instr) && self.program_counter != location
        } { steps += 1; }

        steps
    }

    pub fn jump_to(&mut self, location: i16) {
        self.program_counter = location;
    }

    pub fn fetch(&self) -> i16 {
        self.memory[self.program_counter as usize]
    }

    pub fn decode(memory: i16) -> Instruction {
        Instruction::from(memory) 
    }

    pub fn execute(&mut self, instr: Instruction) -> bool {
        false
    }

    pub fn get_register(&self, idx: usize) -> i16 {
        self.registers[idx]
    }
    pub fn get_program_counter(&self) -> i16 {
        self.program_counter
    }
    pub fn get_condition_code(&self) -> i16 {
        self.state
    }

    pub fn get_memory_location(&self, loc: u16) -> i16 {
        self.memory[loc as usize]
    }
    pub fn get_memory(&self) -> &Box<[i16; 0xFFFF]> {
        return &self.memory
    }

    pub fn get_annotation_location(&self, loc: u16) -> &Option<String> {
        &self.annotations[loc as usize]
    }
    pub fn get_annotation(&self) -> &Box<[Option<String>; 0xFFFF]> {
        return &self.annotations
    }
}
