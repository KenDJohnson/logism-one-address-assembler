use std::fmt;

pub type Immediate = i8;
pub type Address = u8;

#[derive(Debug)]
pub enum Instruction<'a> {
    Add(&'a str),
    AddImmediate(Immediate),
    Subtract(&'a str),
    SubtractImmediate(Immediate),
    Multiply(&'a str),
    MultiplyImmediate(Immediate),
    Divide(&'a str),
    DivideImmediate(Immediate),
    Remainder(&'a str),
    RemainderImmediate(Immediate),
    Shift(Immediate),
    And(&'a str),
    AndImmediate(Immediate),

    BranchZero(&'a str),
    Branch(&'a str),
    ClearAc,
    Store(&'a str),
    NoOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressedInstruction {
    Add(Address),
    AddImmediate(Immediate),
    Subtract(Address),
    SubtractImmediate(Immediate),
    Multiply(Address),
    MultiplyImmediate(Immediate),
    Divide(Address),
    DivideImmediate(Immediate),
    Remainder(Address),
    RemainderImmediate(Immediate),
    Shift(Immediate),
    And(Address),
    AndImmediate(Immediate),
    BranchZero(Address),
    Branch(Address),
    ClearAc,
    Store(Address),
    NoOp,
}

impl AddressedInstruction {
    pub fn opcode(&self) -> u8 {
        match self {
            Self::NoOp => 0,
            Self::AddImmediate(_)
            | Self::SubtractImmediate(_)
            | Self::MultiplyImmediate(_)
            | Self::DivideImmediate(_)
            | Self::AndImmediate(_)
            | Self::RemainderImmediate(_)
            | Self::Shift(_) => 1,
            Self::Add(_)
            | Self::Subtract(_)
            | Self::Multiply(_)
            | Self::Divide(_)
            | Self::Remainder(_)
            | Self::And(_) => 2,
            Self::ClearAc => 3,
            Self::Store(_) => 4,
            Self::BranchZero(_) => 5,
            Self::Branch(_) => 6,
        }
    }

    pub fn alu_op(&self) -> u8 {
        match self {
            Self::NoOp | Self::ClearAc | Self::Store(_) | Self::BranchZero(_) | Self::Branch(_) => {
                0
            }

            Self::AddImmediate(_) | Self::Add(_) => 0,
            Self::SubtractImmediate(_) | Self::Subtract(_) => 1,
            Self::MultiplyImmediate(_) | Self::Multiply(_) => 2,
            Self::DivideImmediate(_) | Self::Divide(_) => 3,
            Self::RemainderImmediate(_) | Self::Remainder(_) => 4,
            Self::AndImmediate(_) | Self::And(_) => 5,
            Self::Shift(_) => 6,
        }
    }

    pub fn value(&self) -> u8 {
        match self {
            Self::NoOp | Self::ClearAc => 0,
            Self::AddImmediate(i)
            | Self::SubtractImmediate(i)
            | Self::MultiplyImmediate(i)
            | Self::DivideImmediate(i)
            | Self::AndImmediate(i)
            | Self::RemainderImmediate(i)
            | Self::Shift(i) => *i as u8,
            Self::Add(i)
            | Self::Subtract(i)
            | Self::Multiply(i)
            | Self::Divide(i)
            | Self::And(i)
            | Self::Store(i)
            | Self::Remainder(i)
            | Self::Branch(i)
            | Self::BranchZero(i) => *i,
        }
    }

    pub fn bytes(&self) -> [u8; 2] {
        let opcode = self.opcode();
        let alu_op = self.alu_op();
        let value = self.value();

        [(opcode << 4) | alu_op, value]
    }

    #[allow(dead_code)]
    pub fn hex_string(&self) -> String {
        let bytes = self.bytes();
        format!("{:02x}{:02x}", bytes[0], bytes[1])
    }
}

impl fmt::Display for AddressedInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Add(addr) => write!(f, "add {:#x}", addr),
            Self::Subtract(addr) => write!(f, "sub {:#x}", addr),
            Self::Multiply(addr) => write!(f, "mul {:#x}", addr),
            Self::Divide(addr) => write!(f, "div {:#x}", addr),
            Self::Remainder(addr) => write!(f, "rem {:#x}", addr),
            Self::And(addr) => write!(f, "and {:#x}", addr),
            Self::Store(addr) => write!(f, "store {:#x}", addr),
            Self::AddImmediate(i) => write!(f, "addi {}", i),
            Self::SubtractImmediate(i) => write!(f, "subi {}", i),
            Self::MultiplyImmediate(i) => write!(f, "muli {}", i),
            Self::DivideImmediate(i) => write!(f, "divi {}", i),
            Self::RemainderImmediate(i) => write!(f, "remi {}", i),
            Self::Shift(i) => write!(f, "shift {}", i),
            Self::AndImmediate(i) => write!(f, "andi {}", i),
            Self::BranchZero(i) => write!(f, "beqz {:#x}", i),
            Self::Branch(i) => write!(f, "br {:#x}", i),
            Self::ClearAc => write!(f, "clac"),
            Self::NoOp => write!(f, "noop"),
        }
    }
}
