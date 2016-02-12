use opcode::OpCode;
use std::fmt;

pub struct Instruction {
    pub opcode: OpCode,
    pub operand: u16,
}

impl fmt::Debug for Instruction {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use opcode::Addressing::*;
        match self.opcode.bytes {
            1 => write!(fmt, "{:?}", self.opcode.id),
            x => {
                let width = (x - 1) * 2;
                match self.opcode.addressing {
                    Absolute => write!(fmt, "{0:01$x}", self.operand, width as usize),
                    /*AbsoluteX,
                    AbsoluteY,
                    Immediate,
                    Indirect,
                    IndirectX,
                    IndirectY,
                    Relative,
                    ZeroPage,
                    ZeroPageX,
                    ZeroPageY,*/
                    _ => write!(fmt, "{:?}", self.opcode.id),
                }
            }
        }
    }
}
