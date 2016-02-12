use bus;
use instruction::Instruction;
use opcode::OpCode;

pub const STACK_BASE: u16 = 0x0100;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StatusBit {
    Carry = 0,
    Zero = 1,
    Interrupt = 2,
    Decimal = 3,
    Break = 4,
    Overflow = 5,
    Negative = 6,
}

#[derive(Debug)]
pub struct Cpu {
    pc: u16,
    ac: u8,
    x: u8,
    y: u8,
    sp: u8,
    sr: u8,
    bus: bus::Bus,
}

impl Cpu {
    pub fn attach_backend(&mut self, entry: bus::BusEntry) {
        self.bus.attach(entry);
    }
    
    pub fn reset(&mut self) {
        self.pc = self.bus.read_u16(0xFFFC);
        self.sr = 0x34;
    }
    
    pub fn step(&mut self) {
        let instruction = self.current_instruction();
        self.pc += instruction.opcode.bytes as u16;
        self.execute(&instruction);
    }
    
    fn execute(&mut self, instruction: &Instruction) {
        use opcode::OpId::*;
        match instruction.opcode.id {
            ADC => self.adc(instruction),
            JMP => self.jmp(instruction),
            _ => panic!("{:?} is not yet implemented", instruction.opcode.id),
        }
    }
    
    fn current_instruction(&self) -> Instruction {
        let code = self.bus.read(self.pc);
        let opcode = match OpCode::get(code) {
            Some(opcode) => opcode,
            invalid => {
                invalid.expect(&format!("Invalid opcode at {:#x}: {:#x}", self.pc, code));
                unreachable!()
            },
        };
        match opcode.bytes {
            1 => Instruction { opcode: opcode, operand: 0 },
            2 => Instruction { opcode: opcode, operand: self.bus.read(self.pc + 1) as u16 },
            3 => Instruction { opcode: opcode, operand: self.bus.read_u16(self.pc + 1) },
            _ => unreachable!(),
        }
    }
    
    fn resolve_operand(&self, instruction: &Instruction) -> u8 {
        use opcode::Addressing::*;
        match instruction.opcode.addressing {
            Immediate => instruction.operand as u8,
            _ => self.bus.read(self.resolve_address(instruction))
        }
    }
    
    fn resolve_address(&self, instruction: &Instruction) -> u16 {
        use opcode::Addressing::*;
        use std::num::Wrapping;
        
        match instruction.opcode.addressing {
            Absolute => instruction.operand,
            AbsoluteX => instruction.operand + self.x as u16,
            AbsoluteY => instruction.operand + self.y as u16,
            Indirect => {
                self.bus.read_u16(instruction.operand)
            },
            IndirectX => {
                let op = Wrapping(instruction.operand as u8);
                let offset = Wrapping(self.x);
                let ptr = (op + offset).0 as u16;
                self.bus.read_u16(ptr)
            },
            IndirectY => {
                self.bus.read_u16(instruction.operand) + self.y as u16
            },
            ZeroPage => instruction.operand,
            ZeroPageX => {
                let op = Wrapping(instruction.operand as u8);
                let offset = Wrapping(self.x);
                (op + offset).0 as u16
            },
            ZeroPageY => {
                let op = Wrapping(instruction.operand as u8);
                let offset = Wrapping(self.y);
                (op + offset).0 as u16
            },
            Relative => {
                let op = instruction.operand as u8 as i8;
                (self.pc as i32 + op as i32) as u16
            },
            m => panic!("Cannot get address with mode `{:?}`", m),
        }
    }
    
    fn status(&self, bit: StatusBit) -> bool {
        self.status_u8(bit) == 1
    }
    
    fn status_u8(&self, bit: StatusBit) -> u8 {
        (self.sr >> (bit as u8)) & 1
    }
    
    fn set_status(&mut self, bit: StatusBit, state: bool) {
        if state {
            self.sr |= 1 << (bit as u8);
        } else {
            self.sr &= !(1 << (bit as u8));
        }
    }
    
    fn update_status(&mut self, value: u8) {
        self.set_status(StatusBit::Zero, value == 0);
        self.set_status(StatusBit::Negative, (value as i8) < 0);
    }
    
    //-------------------------------------------------------
    // Opcode implementation
    
    fn adc(&mut self, instruction: &Instruction) {
        let ac = self.ac as u16;
        let op = self.resolve_operand(instruction) as u16;
        let sr = self.status_u8(StatusBit::Carry) as u16;
        
        let value16 = ac + op + sr;
        self.set_status(StatusBit::Carry, value16 > 0xFF);
        self.ac = value16 as u8;
        self.update_status(value16 as u8);
    }
    
    fn jmp(&mut self, instruction: &Instruction) {
        self.pc = self.resolve_address(instruction);
    }
}
