use bitflags::bitflags;

#[derive(Debug, Clone, Copy)]
struct Registers {
    /* A */ pub accumulator: u8,
    /* F */ pub flags: Flags,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    /* PC */ pub program_counter: u16,
    /* SP */ pub stack_pointer: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Self {
            accumulator: 0,
            flags: Flags::empty(),
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
        use Register8::*;
        match reg {
            B => self.b = value,
            C => self.c = value,
            D => self.d = value,
            E => self.e = value,
            H => self.h = value,
            L => self.l = value,
        }
    }

    pub fn get_reg8(&self, reg: Register8) -> u8 {
        use Register8::*;
        match reg {
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            H => self.h,
            L => self.l,
        }
    }

    pub fn set_reg16(&mut self, reg: Register16, value: u16) {
        use Register16::*;
        match reg {
            BC => {
                let (b, c) = bit16_destructure(value);
                self.b = b;
                self.c = c;
            }
            DE => {
                let (d, e) = bit16_destructure(value);
                self.d = d;
                self.e = e;
            }
            HL => {
                let (h, l) = bit16_destructure(value);
                self.h = h;
                self.l = l;
            }
            StackPointer => self.stack_pointer = value,
        }
    }

    pub fn get_reg16(&self, reg: Register16) -> u16 {
        use Register16::*;
        match reg {
            BC => bit16_structure(self.b, self.c),
            DE => bit16_structure(self.d, self.e),
            HL => bit16_structure(self.h, self.l),
            StackPointer => self.stack_pointer,
        }
    }
}

fn add16(a: u16, b: u16) -> (u16, Flagger) {
    let (result, overflow) = a.overflowing_add(b);

    let mut values = Flags::Subtract;
    if overflow {
        values |= Flags::Carry;
    }

    let flags = Flagger {
        values,
        mask: Flags::Carry | Flags::Subtract,
    };

    (result, flags)
}

fn sub16(a: u16, b: u16) -> (u16, Flagger) {
    let (result, overflow) = a.overflowing_sub(b);

    let mut values = Flags::empty();
    if overflow {
        values |= Flags::Carry;
    }

    let flags = Flagger {
        values,
        mask: Flags::Carry | Flags::Subtract,
    };

    (result, flags)
}

fn bit16_destructure(value: u16) -> (u8, u8) {
    let high = ((value & 0xFF00) >> 8) as u8;
    let low = (value & 0x00FF) as u8;

    (high, low)
}

fn bit16_structure(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | (low as u16)
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

    pub fn execute_operation(&mut self, op: Operation) {
        use Operation::*;

        match op {
            Nop => {}
            Increment(reg) => {
                use Target::*;
                match reg {
                    Bit8(reg) => {
                        let read = self.registers.get_reg8(reg);
                        let (write, _) = read.overflowing_add(1);
                        self.registers.set_reg8(reg, write);
                    }
                    Bit16(reg) => {
                        let read = self.registers.get_reg16(reg);
                        let (write, _) = read.overflowing_add(1);
                        self.registers.set_reg16(reg, write);
                    }
                }
            }

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
    StackPointer,
}

#[derive(Debug, Clone, Copy)]
pub enum Target {
    Bit8(Register8),
    Bit16(Register16),
}

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    /* NOP  */ Nop,
    /* STOP */ Stop,
    /* LD   */ Load,
    Add,
    Sub,
    Increment(Target),
    Decrement,
}

impl Operation {
    pub fn parse(bytecode: [u8; 4]) -> Option<Operation> {
        let instruction = bytecode[0];

        match instruction {
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FlagFilterType {
    Operation,
    Set,
    Reset,
}

#[derive(Debug, Clone, Copy)]
pub struct FlagFilter {
    zero: Option<FlagFilterType>,
    subtract: Option<FlagFilterType>,
    halfcarry: Option<FlagFilterType>,
    carry: Option<FlagFilterType>,
}

impl FlagFilter {
    pub fn filter(&self, src: Flags, dst: Flags) -> Flags {
        unimplemented!()
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct Flags: u8 {
        const Zero = 0b1000_0000;
        const Subtract = 0b0100_0000;
        const HalfCarry = 0b0010_0000;
        const Carry = 0b0001_0000;
    }
}

pub struct Flagger {
    values: Flags,
    mask: Flags,
}

impl Flagger {
    pub fn new() -> Self {
        Self {
            values: Flags::empty(),
            mask: Flags::empty(),
        }
    }
}
