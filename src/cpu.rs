#[derive(Clone, Copy, Debug)]
enum Register {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
}

impl Register {
    fn new(index: u32) -> Self {
        match index & 0b111 {
            0b000 => Self::R0,
            0b001 => Self::R1,
            0b010 => Self::R2,
            0b011 => Self::R3,
            0b100 => Self::R4,
            0b101 => Self::R5,
            0b110 => Self::R6,
            0b111 => Self::R7,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
enum Instruction {
    Add {
        reg_a: Register,
        reg_b: Register,
        dst_reg: Register,
    },
    Nor {
        reg_a: Register,
        reg_b: Register,
        dst_reg: Register,
    },
    Lw {
        reg_a: Register,
        reg_b: Register,
        offset_field: i16,
    },
    Sw {
        reg_a: Register,
        reg_b: Register,
        offset_field: i16,
    },
    Beq {
        reg_a: Register,
        reg_b: Register,
        offset_field: i16,
    },
    Jalr {
        reg_a: Register,
        reg_b: Register,
    },
    Halt,
    Noop,
}

impl Instruction {
    fn new(code: u32) -> Self {
        match code >> 22 & 0b111 {
            0b000 => {
                let (reg_a, reg_b, dst_reg) = Instruction::parse_r(code);
                Self::Add {
                    reg_a,
                    reg_b,
                    dst_reg,
                }
            }
            0b001 => {
                let (reg_a, reg_b, dst_reg) = Instruction::parse_r(code);
                Self::Nor {
                    reg_a,
                    reg_b,
                    dst_reg,
                }
            }
            0b010 => {
                let (reg_a, reg_b, offset_field) = Instruction::parse_i(code);
                Self::Lw {
                    reg_a,
                    reg_b,
                    offset_field,
                }
            }
            0b011 => {
                let (reg_a, reg_b, offset_field) = Instruction::parse_i(code);
                Self::Sw {
                    reg_a,
                    reg_b,
                    offset_field,
                }
            }
            0b100 => {
                let (reg_a, reg_b, offset_field) = Instruction::parse_i(code);
                Self::Beq {
                    reg_a,
                    reg_b,
                    offset_field,
                }
            }
            0b101 => {
                let (reg_a, reg_b) = Instruction::parse_j(code);
                Self::Jalr { reg_a, reg_b }
            }
            0b110 => Self::Halt,
            0b111 => Self::Noop,
            _ => unreachable!(),
        }
    }

    fn parse_r(code: u32) -> (Register, Register, Register) {
        (
            Register::new(code >> 19),
            Register::new(code >> 16),
            Register::new(code),
        )
    }

    fn parse_i(code: u32) -> (Register, Register, i16) {
        (
            Register::new(code >> 19),
            Register::new(code >> 16),
            (code & 0xFFFF) as i16,
        )
    }

    fn parse_j(code: u32) -> (Register, Register) {
        (Register::new(code >> 19), Register::new(code >> 16))
    }
}

const MEMORY_SIZE: usize = 65536;
#[derive(Debug)]
pub struct CPU {
    register_file: [u32; 8],
    memory: Box<[u32; MEMORY_SIZE]>,
    pc: u32,
    halted: bool,
}

impl CPU {
    pub fn new<T: Iterator<Item = u32>>(starting_memory: T) -> Self {
        let mut memory = Box::new([0; MEMORY_SIZE]);
        for (index, item) in starting_memory.enumerate() {
            memory[index] = item;
        }
        CPU {
            register_file: [0; 8],
            memory,
            pc: 0,
            halted: false,
        }
    }

    pub fn print_registers(&self) {
        for i in 0..8 {
            println!("R{i}: {}", self.register_file[i]);
        }
    }

    pub fn print_memory(&self, start_addr: u32, count: u32) {
        for val in &self.memory[(start_addr as usize)..(start_addr as usize) + (count as usize)] {
            print!("{:08X} ", val);
        }
        println!();
    }

    pub fn print_program_counter(&self) {
        println!("{}", self.pc);
    }

    #[inline]
    fn get_register(&self, register: Register) -> u32 {
        self.register_file[register as usize]
    }

    #[inline]
    fn set_register(&mut self, register: Register, value: u32) {
        self.register_file[register as usize] = value;
    }

    /// Returns true if the program has halted
    pub fn step_n(&mut self, count: usize) -> bool {
        if self.halted {
            return true;
        };
        for _ in 0..count {
            if self.step() {
                self.halted = true;
                return true;
            }
        }
        false
    }

    /// Returns true if the program has halted
    pub fn step(&mut self) -> bool {
        if self.halted {
            return true;
        };
        let instruction = self.memory[self.pc as usize];
        let instruction = Instruction::new(instruction);
        match instruction {
            Instruction::Add {
                reg_a,
                reg_b,
                dst_reg,
            } => {
                self.set_register(
                    dst_reg,
                    self.get_register(reg_a)
                        .wrapping_add(self.get_register(reg_b)),
                );
            }
            Instruction::Nor {
                reg_a,
                reg_b,
                dst_reg,
            } => {
                self.set_register(
                    dst_reg,
                    !(self.get_register(reg_a) | self.get_register(reg_b)),
                );
            }
            Instruction::Lw {
                reg_a,
                reg_b,
                offset_field,
            } => {
                self.set_register(
                    reg_b,
                    self.memory
                        [CPU::offset_memory(self.get_register(reg_a), offset_field) as usize],
                );
            }
            Instruction::Sw {
                reg_a,
                reg_b,
                offset_field,
            } => {
                self.memory[CPU::offset_memory(self.get_register(reg_a), offset_field) as usize] =
                    self.get_register(reg_b);
            }
            Instruction::Beq {
                reg_a,
                reg_b,
                offset_field,
            } => {
                if self.get_register(reg_a) == self.get_register(reg_b) {
                    self.pc = CPU::offset_memory(self.pc, offset_field);
                }
            }
            Instruction::Jalr { reg_a, reg_b } => {
                self.set_register(reg_b, self.pc + 1);
                self.pc = self.get_register(reg_a);
                return false;
            }
            Instruction::Halt => {
                self.pc += 1;
                self.halted = true;
                return true;
            }
            Instruction::Noop => {}
        }
        self.pc += 1;
        false
    }

    fn offset_memory(address: u32, offset_field: i16) -> u32 {
        address.wrapping_add_signed(offset_field as i32)
    }
}
