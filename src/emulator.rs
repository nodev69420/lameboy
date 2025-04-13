#[derive(Debug, Clone, Copy)]
struct Registers {
    /* a */ pub accumulator: u8,
    /* f */ pub flags: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    /* pc */ pub program_counter: u16,
    /* sp */ pub stack_pointer: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Self {
            accumulator: 0,
            flags: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            program_counter: 0,
            stack_pointer: 0,
        }
    }

    pub fn set_reg8(&mut self, reg: Register8, value: u8) {
        unimplemented!()
    }

    pub fn get_reg8(&self, reg: Register8) -> u8 {
        unimplemented!()
    }

    pub fn set_reg16(&mut self, reg: Register16, value: u16) {
        unimplemented!()
    }

    pub fn get_reg16(&self, reg: Register16) -> u16 {
        unimplemented!()
    }
}

pub struct MemoryMap {}

impl MemoryMap {
    pub fn write8(&mut self, address: u16, value: u8) {
        unimplemented!()
    }

    pub fn write16(&mut self, address: u16, value: u16) {
        unimplemented!()
    }

    pub fn read8(&self, address: u16) -> u8 {
        unimplemented!()
    }

    pub fn read16(&self, address: u16) -> u16 {
        unimplemented!()
    }
}

struct Gameboy {
    registers: Registers,
    memory: MemoryMap,
}

impl Gameboy {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            memory: MemoryMap {},
        }
    }

    pub fn execute_instruction(&mut self, instruction: Instruction) {
        use Instruction::*;

        match instruction {
            Nop => {}
            Increment(reg) => {
                use AnyRegister::*;
                match reg {
                    Bit8(reg) => {
                        let read = self.registers.get_reg8(reg);
                        let (write, _) = read.overflowing_add(1);
                        self.registers.set_reg8(reg, write);
                    },
                    Bit16(reg) => {
                        let read = self.registers.get_reg16(reg);
                        let (write, _) = read.overflowing_add(1);
                        self.registers.set_reg16(reg, write);
                    }
                }
            },
            
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Register8 {
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug, Clone, Copy)]
pub enum Register16 {
    BC,
    DE,
    HL,
}

#[derive(Debug, Clone, Copy)]
pub enum AnyRegister {
    Bit8(Register8),
    Bit16(Register16),
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    /* NOP  */ Nop,
    /* STOP */ Stop,
    /* LD   */ Load,
    Add,
    Sub,
    Increment(AnyRegister),
    Decrement,
}

impl Instruction {
    pub fn parse(bytecode: [u8; 4]) -> Option<Instruction> {
        let instruction = bytecode[0];

        match instruction {
            
            _ => None,
        }
    }
}
