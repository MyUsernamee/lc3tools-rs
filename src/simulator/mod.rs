use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc, sync::{Arc, Mutex}};

pub mod consts;
pub mod instruction;
pub mod registers;
pub mod utils;

pub use consts::*;
pub use instruction::*;
pub use registers::*;
pub use utils::*;

#[derive(Debug)]
pub enum ObjLoadErr {
    MissingData,
    MagicHeader { expected: [u8; 5], got: [u8; 5] },
    VersionString { expected: [u8; 2], got: [u8; 2] },
}

pub struct LC3Simulator {
    registers: [u16; 8],
    program_counter: u16,
    user_mode: bool,
    priority: u16,
    state: (bool, bool, bool),
    memory: Box<[u16; 0x10000]>,
    annotations: Box<[Option<String>; 0x10000]>,
    supervisor_stack_pointer: u16,
    user_stack_pointer: u16,
    write_callbacks: HashMap<u16, Arc<Mutex<dyn Fn(&mut LC3Simulator, u16) + Sync + Send>>>,
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
            .field(
                "annotations[pc]",
                &self.annotations[self.program_counter as usize],
            )
            .field("supervisor_stack_pointer", &self.supervisor_stack_pointer)
            .field("user_stack_pointer", &self.user_stack_pointer)
            .finish()
    }
}

impl Default for LC3Simulator {
    /// Default LC3Simulator. All registers, program counter, priority
    /// , and memory are all 0. Starts in user (unpriviledged) mode.
    /// empty state / PSR register, and no annotations.
    /// Supervisor stack pointer is initialized to 0x3000, and user
    /// stack pointer is initialized to 0xFEFF
    fn default() -> Self {
        let mut ret = LC3Simulator {
            registers: [0; 8],
            program_counter: 0,
            user_mode: true,
            priority: 0,
            state: (false, false, false),
            memory: Box::new([0; 0x10000]),
            annotations: Box::new([const { None }; 0x10000]),
            supervisor_stack_pointer: 0x3000,
            user_stack_pointer: 0xFDFF,
            write_callbacks: HashMap::new(),
        };
        ret.memory[0xFFFE] = 1 << 15;
        ret
    }
}

impl LC3Simulator {
    /// Creates a new default LC3Simulator
    pub fn new() -> LC3Simulator {
        Self::default()
    }

    /// Creates a new LC3Simulator with the default LC3 OS.
    pub fn with_os() -> LC3Simulator {
        let mut sim = LC3Simulator::new();
        sim.load_obj(include_bytes!("../../lc3os/os.obj").to_vec(), true)
            .expect("Failed to load OS.");
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
                return Err(ObjLoadErr::MagicHeader {
                    expected: *MAGIC_HEADER,
                    got: data[0..5].try_into().unwrap(),
                });
            }
            ptr += 1
        }
        for byte in VERSION_STRING {
            if *byte != data[ptr] {
                return Err(ObjLoadErr::VersionString {
                    expected: *VERSION_STRING,
                    got: data[5..7].try_into().unwrap(),
                });
            }
            ptr += 1
        }

        let mut orig = 0;
        let mut mem_ptr = 0;
        while ptr < data.len() {
            let word = u16::from_ne_bytes(data[ptr..ptr + 2].try_into().unwrap());
            ptr += 2;

            let is_orig = data[ptr] == 1;
            ptr += 1;

            let annotation_size = u32::from_ne_bytes(data[ptr..ptr + 4].try_into().unwrap());
            ptr += 4;

            let mut annotation = None;
            if annotation_size > 0 {
                annotation = Some(
                    String::from_utf8(
                        data[(ptr as usize)..((ptr as u32 + annotation_size) as usize)].to_vec(),
                    )
                    .unwrap(),
                );
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
    pub fn step(&mut self) -> bool {
        let memory = self.fetch();
        let instr = LC3Simulator::decode(memory);
        self.execute(instr)
    }
    pub fn reset(&mut self) {
        self.memory[0xFFFE] = 1 << 15;
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
                    Operand::Register(reg) => self.registers[reg as usize],
                };
                let result = self.registers[sr1 as usize].wrapping_add(other);
                self.update_condition_code(result);
                self.registers[dr as usize] = result;
            }
            Instruction::AND { dr, sr1, op } => {
                let result = self.registers[sr1 as usize]
                    & match op {
                        Operand::Immediate(value) => value,
                        Operand::Register(reg) => self.registers[reg as usize],
                    };
                self.update_condition_code(result);
                self.registers[dr as usize] = result;
            }
            Instruction::BR { n, z, p, offset } => {
                let (s_n, s_z, s_p) = self.state;
                if (n & s_n) | (z & s_z) | (p & s_p) | (n & z & p) {
                    self.program_counter =
                        self.program_counter.wrapping_add(sign_extend(9, offset));
                }
            }
            Instruction::JMP { base_r } => {
                self.program_counter = self.registers[base_r as usize];
            }
            Instruction::JSR { op } => {
                self.program_counter = match op {
                    Operand::Register(reg) => self.registers[reg as usize],
                    Operand::Immediate(offset) => sign_extend(11, offset),
                }
            }
            i @ (Instruction::LD { dr, offset, .. }
            | Instruction::LDI { dr, offset, .. }
            | Instruction::LDR { dr, offset, .. }) => {
                let addr = match i {
                    Instruction::LD { .. } => self.program_counter + sign_extend(9, offset),
                    Instruction::LDI { .. } => {
                        self.memory[(self.program_counter + sign_extend(9, offset)) as usize]
                    }
                    Instruction::LDR { base_r, .. } => {
                        self.registers[base_r as usize] + sign_extend(6, offset)
                    }
                    _ => 0,
                };
                let result = self.memory[addr as usize];
                self.update_condition_code(result);
                self.registers[dr as usize] = result;
            }
            Instruction::LEA { dr, offset } => {
                let result = self.program_counter.wrapping_add(sign_extend(9, offset));
                self.registers[dr as usize] = result;
            }
            Instruction::NOT { dr, sr } => {
                let result = !self.registers[sr as usize];
                self.update_condition_code(result);
                self.registers[dr as usize] = result;
            }
            Instruction::RTI => {
                if self.user_mode {
                    dbg!(&self, format!("{:b}", self.read(self.program_counter)));
                    todo!();
                } else {
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
            }
            i @ (Instruction::ST { sr, offset }
            | Instruction::STI { sr, offset }
            | Instruction::STR { sr, offset, .. }) => {
                let location = match i {
                    Instruction::ST { .. } => {
                        self.program_counter.wrapping_add(sign_extend(9, offset))
                    }
                    Instruction::STI { .. } => {
                        self.memory
                            [(self.program_counter.wrapping_add(sign_extend(9, offset))) as usize]
                    }
                    Instruction::STR { base_r, .. } => {
                        self.registers[base_r as usize].wrapping_add(sign_extend(6, offset))
                    }
                    _ => 0,
                };

                if self.user_mode && location < 0x3000 {
                    self.interrupt(0x02);
                } else {
                    self.write(self.registers[sr as usize], location);
                }
            }
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
            }
            Instruction::NOOP => {}
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
        let state =
            ((self.state.2 as u16) << 2) | ((self.state.1 as u16) << 1) | (self.state.0 as u16);
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

    pub fn get_memory(&self) -> &Box<[u16; 0x10000]> {
        return &self.memory;
    }

    pub fn read(&self, loc: u16) -> u16 {
        return self.memory[loc as usize];
    }

    pub fn write(&mut self, value: u16, location: u16) {
        self.memory[location as usize] = value;
        let callback = self.write_callbacks.get(&location);
        if callback.is_none() {
            return;
        }
        let callback = callback.unwrap().clone();
        callback.lock().unwrap()(self, value);
    }

    pub fn add_write_callback<T: Fn(&mut LC3Simulator, u16) + 'static + Sync + Send>(
        &mut self,
        loc: u16,
        callback: T,
    ) {
        self.write_callbacks
            .insert(loc, Arc::new(Mutex::new(callback)));
    }

    pub fn get_annotation_location(&self, loc: u16) -> &Option<String> {
        &self.annotations[loc as usize]
    }
    pub fn get_annotations(&self) -> &Box<[Option<String>; 0x10000]> {
        return &self.annotations;
    }

    pub fn set_register<T: Into<usize>>(&mut self, reg: T, value: u16) {
        self.registers[reg.into()] = value;
    }
}
