use std::{cell::RefCell, collections::HashMap, fmt::Debug, iter::zip, ops::Range, rc::Rc, slice::Iter, str::Bytes};

// std::string lc3::utils::getMagicHeader(void) { return "\x1c\x30\x15\xc0\x01"; }
// std::string lc3::utils::getVersionString(void) { return "\x01\x01"; }

const MAGIC_HEADER: &[u8; 5] = b"\x1c\x30\x15\xc0\x01";
const VERSION_STRING: &[u8; 2] = b"\x01\x01";

pub fn sign_extend(bits: u16, value: u16) -> u16 {
    let value = value & ((1 << bits) - 1);
    let sign_bit = value & (1 << bits - 1);
    if sign_bit != 0 {
        return (!(0b0u16) << bits) | value;
    }
    value
}

fn get_bit(value: u16, bit: u16) -> bool {
    (value >> bit) == 1
}
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
            _ => Register::R0
        }
    }
}

impl Into<usize> for Register {
    fn into(self) -> usize {
        return self as usize
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operand {
    Immediate(u16),
    Register(Register)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    ADD {dr: Register, sr1: Register, op: Operand},
    AND {dr: Register, sr1: Register, op: Operand},
    BR { n: bool, z: bool, p: bool, offset: u16 },
    JMP {base_r: Register},
    JSR {op: Operand},
    LD {dr: Register, offset: u16},
    LDI {dr: Register, offset: u16},
    LDR {dr: Register, base_r: Register, offset: u16},
    LEA {dr: Register, offset: u16},
    NOT {dr: Register, sr: Register},
    RTI,
    ST {sr: Register, offset: u16},
    STI {sr: Register, offset: u16},
    STR {sr: Register, base_r: Register, offset: u16},
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
                op: if (value >> 5) & 0b1 == 1 { Operand::Immediate(sign_extend(5, value)) } else {Operand::Register(Register::from(value & 0b111))}
            },
            0b0101 => Instruction::AND {
                dr: Register::from((value >> 9) & 0b111),
                sr1: Register::from((value >> 6) & 0b111),
                op: if (value >> 5) & 0b1 == 1 { Operand::Immediate(sign_extend(5, value )) } else {Operand::Register(Register::from(value & 0b111))}
            },
            0b0000 => Instruction::BR { 
                n: (value >> 11) & 0b1 == 1, 
                z: (value >> 10) & 0b1 == 1, 
                p: (value >> 9) & 0b1 == 1, 
                offset: sign_extend(9, value)
            },
            0b1100 => Instruction::JMP { base_r: Register::from((value >> 6) & 0b111) }, 
            0b0100 => Instruction::JSR { op: if ((value >> 11) & 0b1) == 1 { Operand::Immediate(sign_extend(10, value)) } else {Operand::Register(Register::from((value >> 6) & 0b111))} },
            0b0010 => Instruction::LD { dr: Register::from((value >> 9) & 0b111), offset: sign_extend(9, value) },
            0b1010 => Instruction::LDI { dr: Register::from((value >> 9) & 0b111), offset: sign_extend(9, value) },
            0b0110 => Instruction::LDR { dr: Register::from((value >> 9) & 0b111), base_r: Register::from((value >> 6) & 0b111), offset: sign_extend(6, value) },
            0b1110 => Instruction::LEA { dr: Register::from((value >> 9) & 0b111), offset: sign_extend(9, value) },
            0b1001 => Instruction::NOT { dr: Register::from((value >> 9) & 0b111), sr: Register::from((value >> 6) & 0b111) },
            0b1000 => Instruction::RTI,
            0b0011 => Instruction::ST { sr: Register::from((value >> 9) & 0b111), offset: sign_extend(9, value) },
            0b1011 => Instruction::STI { sr: Register::from((value >> 9) & 0b111), offset: sign_extend(9, value) },
            0b0111 => Instruction::STR { sr: Register::from((value >> 9) & 0b111), base_r: Register::from((value >> 6) & 0b111), offset: sign_extend(6, value) },
            0b1111 => Instruction::TRAP(value as u8),
            _ => Instruction::NOOP
        }
    }
}

pub struct LC3Simulator {
    registers: [u16; 8],
    program_counter: u16,
    user_mode: bool,
    priority: u16,
    state: (bool, bool, bool),
    memory: Box<[u16; 0xFFFF]>,
    annotations: Box<[Option<String>; 0xFFFF]>,
    supervisor_stack_pointer: u16,
    user_stack_pointer: u16,
    write_callbacks: HashMap<u16, Box<dyn FnMut(u16) -> ()>>
}

impl Debug for LC3Simulator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("LC3Simulator")
            .field("registers", &self.registers)
            .field("program_counter", &self.program_counter)
            .field("user_mode", &self.user_mode)
            .field("priority", &self.priority)
            .field("state", &self.state)
            .field("memory[pc]", &self.memory[self.program_counter as usize])
            .field("annotations[pc]", &self.annotations[self.program_counter as usize])
            .field("supervisor_stack_pointer", &self.supervisor_stack_pointer)
            .field("user_stack_pointer", &self.user_stack_pointer)
            .finish()
    }
}

#[derive(Debug)]
pub enum ObjLoadErr {
    MissingData,
    MagicHeader {expected: [u8; 5], got: [u8; 5]},
    VersionString {expected: [u8; 2], got: [u8; 2]}
}

impl LC3Simulator {
    pub fn new() -> LC3Simulator {
        LC3Simulator { registers: [0; 8], program_counter: 0, user_mode: true, priority: 0, state: (false, false, false), memory: Box::new([0; 0xFFFF]), annotations: Box::new([const { None }; 0xFFFF]), supervisor_stack_pointer: 0x3000, user_stack_pointer: 0, write_callbacks: HashMap::new()}
    }

    pub fn with_os() -> LC3Simulator {
        let mut sim = LC3Simulator::new();
        sim.load_obj(include_bytes!("../lc3os/os.obj").to_vec(), true).expect("Failed to load OS.");
        sim.load_obj(include_bytes!("../lc3os/trap_routines.obj").to_vec(), true).expect("Failed to load OS.");
        sim
    }

    /// Loads obj from vector of bytes.
    /// If jump is true, jump to the origin specified in the obj
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
        let mut mem_ptr = 0;
        let mut words: Vec<u16> = Vec::new();
        let mut annotations: Vec<Option<String>> = Vec::new();
        while ptr < data.len() {
            let word = u16::from_ne_bytes(data[ptr..ptr+2].try_into().unwrap());
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
                mem_ptr = orig;
                continue;
            }

            self.memory[mem_ptr as usize] = word;
            self.annotations[mem_ptr as usize] = annotation.clone();
            mem_ptr += 1;
        }

        if jump {
            self.jump_to(orig);
        }

        Ok(())
    }
    pub fn step (&mut self) -> bool {
        let memory = self.fetch();
        let instr = LC3Simulator::decode(memory);
        self.execute(instr)

    } 
    pub fn reset(&mut self) {
        self.memory[0xFFFE] = 1<<15;
    }

    pub fn jump_to(&mut self, location: u16) {
        self.program_counter = location;
    }

    pub fn fetch(&self) -> u16 {
        self.memory[self.program_counter as usize]
    }

    pub fn decode(memory: u16) -> Instruction {
        Instruction::from(memory) 
    }

    pub fn execute(&mut self, instr: Instruction) -> bool {
        self.program_counter += 1;

        match instr {
            Instruction::ADD { dr, sr1, op } => {
                let other: u16 = match op {
                    Operand::Immediate(value) => value,
                    Operand::Register(reg) => self.registers[reg as usize]
                };
                let result = self.registers[sr1 as usize].wrapping_add(other);
                self.update_condition_code(result);
                self.registers[dr as usize] = result;
            },
            Instruction::AND { dr, sr1, op } => {
                let result = self.registers[sr1 as usize] & match op {
                    Operand::Immediate(value) => value,
                    Operand::Register(reg) => self.registers[reg as usize]
                };
                self.update_condition_code(result);
                self.registers[dr as usize] = result;
            },
            Instruction::BR { n, z, p, offset } => {
                let (N, Z, P) = self.state;
                if (n & N) | (z & Z) | (p & P) {
                    self.program_counter = self.program_counter.wrapping_add(sign_extend(9, offset));
                }
            },
            Instruction::JMP { base_r } => {
                self.program_counter = self.registers[base_r as usize];
            },
            Instruction::JSR { op } => {
                self.program_counter = match op {
                    Operand::Register(reg) => self.registers[reg as usize],
                    Operand::Immediate(offset) => sign_extend(11, offset)
                }
            },
            i @ (Instruction::LD { dr, offset, .. } | Instruction::LDI { dr, offset, .. } | Instruction::LDR { dr, offset, .. }) => {
                let addr = match i {
                    Instruction::LD {..} => self.program_counter + sign_extend(9, offset),
                    Instruction::LDI {..} => self.memory[(self.program_counter + sign_extend(9, offset)) as usize],
                    Instruction::LDR {base_r, ..} => self.registers[base_r as usize] + sign_extend(6, offset),
                    _ => 0
                };
                let result = self.memory[addr as usize];
                self.update_condition_code(result);
                self.registers[dr as usize] = result;
            },
            Instruction::LEA { dr, offset } => {
                let result = self.program_counter.wrapping_add(sign_extend(9, offset));
                self.registers[dr as usize] = result;
            },
            Instruction::NOT { dr, sr } => {
                let result = !self.registers[sr as usize];
                self.update_condition_code(result);
                self.registers[dr as usize] = result;
            },
            Instruction::RTI => {
                if self.user_mode {
                    dbg!(&self, format!("{:b}", self.read(self.program_counter)));
                    todo!()
                }
                else {
                    self.program_counter = self.read(self.registers[6]);
                    self.registers[6] += 1;
                    let temp = self.memory[self.registers[6] as usize];
                    self.registers[6] += 1;
                    self.set_psr(temp);                    
                    if self.user_mode {
                        self.supervisor_stack_pointer = self.registers[6];
                        self.registers[6] = self.user_stack_pointer;
                    }
                }
            },
            i @ (Instruction::ST { sr, offset } | Instruction::STI { sr, offset } |  Instruction::STR { sr, offset, .. }) => {
                let location = match i {
                    Instruction::ST { .. } => self.program_counter.wrapping_add(sign_extend(9, offset)),
                    Instruction::STI { .. } => self.memory[(self.program_counter.wrapping_add(sign_extend(9, offset))) as usize],
                    Instruction::STR { base_r, .. } => self.registers[base_r as usize].wrapping_add(sign_extend(6, offset)),
                    _ => 0
                };

                if self.user_mode && location < 0x3000 {
                    self.interrupt(0x02);
                }
                else {
                    self.write(self.registers[sr as usize], location); 
                }
            },
            Instruction::TRAP(trap_vec) => {
                let psr = self.get_psr();
                if self.user_mode {
                    self.user_stack_pointer = self.registers[6];
                    self.registers[6] = self.supervisor_stack_pointer;
                }
                self.user_mode = false;
                self.registers[6] -= 1;
                self.write(psr, self.registers[6]);
                self.registers[6] -= 1;
                self.write(self.program_counter, self.registers[6]);
                self.program_counter = self.memory[trap_vec as usize];
            },
            Instruction::NOOP => {},
        }

    
        return (self.memory[0xFFFE] >> 15) == 1;
    }

    fn update_condition_code(&mut self, value: u16) {
        let signed = value as i16;
        self.state = match (signed == 0, signed < 0) {
            (true, _) => (self.state.0, true, false),
            (false, true) => (true, self.state.1, false),
            (false, false) => (false, self.state.1, true),
        };
    }

    pub fn interrupt(&mut self, vector: u16) {
        let temp = self.get_psr();
        self.user_mode = false;
        if get_bit(temp, 15) {
            self.user_stack_pointer = self.registers[6];
            self.registers[6] = self.supervisor_stack_pointer;
        }
        self.registers[6] -= 1;
        self.write(temp, self.registers[6]);
        self.registers[6] -= 1;
        self.write(self.program_counter, self.registers[6]);
        self.program_counter = self.memory[(vector + 0x100) as usize];
    }

    pub fn get_psr(&self) -> u16 {
        let state = ((self.state.2 as u16) << 2) | ((self.state.1 as u16) << 1) | (self.state.0 as u16);
        ((self.user_mode as u16) << 15) | self.priority << 3 | state
    }
    pub fn set_psr(&mut self, psr: u16) {
        self.user_mode = get_bit(psr, 15);
        self.priority = psr & 0b1111111000;
        self.state = (get_bit(psr, 2), get_bit(psr, 1), get_bit(psr, 0));
    }
    pub fn get_register(&self, idx: usize) -> u16 {
        self.registers[idx]
    }
    pub fn get_program_counter(&self) -> u16 {
        self.program_counter
    }
    pub fn get_condition_code(&self) -> (bool, bool, bool) {
        self.state
    }

    pub fn get_memory(&self) -> &Box<[u16; 0xFFFF]> {
        return &self.memory
    }

    pub fn read(&self, loc: u16) -> u16 {
        return self.memory[loc as usize]
    }

    pub fn write(&mut self, value: u16, location: u16) {
        self.memory[location as usize] = value;
        if let Some(callback) = self.write_callbacks.get_mut(&location) {
            callback(value);
        }
    }

    pub fn add_write_callback<T: FnMut(u16) -> () + 'static>(&mut self, loc: u16, callback: T) {
        self.write_callbacks.insert(loc, Box::new(callback));
    }

    pub fn get_annotation_location(&self, loc: u16) -> &Option<String> {
        &self.annotations[loc as usize]
    }
    pub fn get_annotations(&self) -> &Box<[Option<String>; 0xFFFF]> {
        return &self.annotations
    }
    
    pub fn set_register<T: Into<usize>>(&mut self, reg: T, value: u16) {
        self.registers[reg.into()] = value;
    }

}
