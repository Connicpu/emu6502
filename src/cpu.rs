use bus;
use instruction::Instruction;
use opcode::OpCode;
use std::num::wrapping::OverflowingOps;

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
    pub fn new() -> Cpu {
        Cpu {
            pc: 0,
            ac: 0,
            x: 0,
            y: 0,
            sp: 0,
            sr: 0,
            bus: bus::Bus::new(),
        }
    }
    
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
    
    pub fn status(&self, bit: StatusBit) -> bool {
        self.status_u8(bit) == 1
    }
    
    pub fn set_status(&mut self, bit: StatusBit, state: bool) {
        if state {
            self.sr |= 1 << (bit as u8);
        } else {
            self.sr &= !(1 << (bit as u8));
        }
    }
    
    fn execute(&mut self, instruction: &Instruction) {
        use opcode::OpId::*;
        let op: fn(&mut Cpu, &Instruction) = match instruction.opcode.id {
            ADC => Cpu::adc,
            AND => Cpu::and,
            ASL => Cpu::asl,
            BCC => Cpu::bcc,
            BCS => Cpu::bcs,
            BEQ => Cpu::beq,
            BIT => Cpu::bit,
            BMI => Cpu::bmi,
            BNE => Cpu::bne,
            BPL => Cpu::bpl,
            BRK => Cpu::brk,
            CLC => Cpu::clc,
            CLD => Cpu::cld,
            CLI => Cpu::cli,
            CMP => Cpu::cmp,
            CPX => Cpu::cpx,
            CPY => Cpu::cpy,
            DEC => Cpu::dec,
            DEX => Cpu::dex,
            DEY => Cpu::dey,
            JMP => Cpu::jmp,
            _ => panic!("{:?} is not yet implemented", instruction.opcode.id),
        };
        
        op(self, instruction)
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
        
        match instruction.opcode.addressing {
            Absolute => instruction.operand,
            AbsoluteX => instruction.operand + self.x as u16,
            AbsoluteY => instruction.operand + self.y as u16,
            Indirect => {
                self.bus.read_u16(instruction.operand)
            },
            IndirectX => {
                let op = instruction.operand as u8;
                let ptr = op.overflowing_add(self.x).0 as u16;
                self.bus.read_u16(ptr)
            },
            IndirectY => {
                self.bus.read_u16(instruction.operand) + self.y as u16
            },
            ZeroPage => instruction.operand,
            ZeroPageX => {
                let op = instruction.operand as u8;
                op.overflowing_add(self.x).0 as u16
            },
            ZeroPageY => {
                let op = instruction.operand as u8;
                op.overflowing_add(self.y).0 as u16
            },
            Relative => {
                let op = instruction.operand;
                self.pc + op as u8 as i8 as i16 as u16
            },
            m => panic!("Cannot get address with mode `{:?}`", m),
        }
    }
    
    fn branch(&mut self, instruction: &Instruction) {
        self.pc += instruction.operand as u8 as i8 as i16 as u16;
    }
    
    fn status_u8(&self, bit: StatusBit) -> u8 {
        (self.sr >> (bit as u8)) & 1
    }
    
    fn update_status(&mut self, value: u8) {
        self.set_status(StatusBit::Zero, value == 0);
        self.set_status(StatusBit::Negative, (value as i8) < 0);
    }
    
    fn update_ac(&mut self) {
        let ac = self.ac;
        self.update_status(ac);
    }
    
    fn update_x(&mut self) {
        let x = self.x;
        self.update_status(x);
    }
    
    fn update_y(&mut self) {
        let y = self.y;
        self.update_status(y);
    }
    
    //-------------------------------------------------------
    // Opcode implementation
    
    pub fn adc(&mut self, instruction: &Instruction) {
        let ac = self.ac as u16;
        let op = self.resolve_operand(instruction) as u16;
        let sr = self.status_u8(StatusBit::Carry) as u16;
        
        let value16 = ac + op + sr;
        self.set_status(StatusBit::Carry, value16 > 0xFF);
        self.ac = value16 as u8;
        self.update_ac();
    }
    
    pub fn and(&mut self, instruction: &Instruction) {
        self.ac &= self.resolve_operand(instruction);
        self.update_ac();
    }
    
    pub fn asl(&mut self, instruction: &Instruction) {
        use opcode::Addressing::*;
        match instruction.opcode.addressing {
            Accumulator => {
                let ac_carry = (self.ac >> 7) == 1;
                self.set_status(StatusBit::Carry, ac_carry);
                self.ac <<= 1;
                self.update_ac();
            },
            _ => {
                let addr = self.resolve_address(instruction);
                let mut value = self.bus.read(addr);
                self.set_status(StatusBit::Carry, (value >> 7) == 1);
                value <<= 1;
                self.bus.write(addr, value);
                self.update_status(value);
            }
        }
    }
    
    pub fn bcc(&mut self, instruction: &Instruction) {
        if !self.status(StatusBit::Carry) {
            self.branch(instruction);
        }
    }
    
    pub fn bcs(&mut self, instruction: &Instruction) {
        if self.status(StatusBit::Carry) {
            self.branch(instruction);
        }
    }
    
    pub fn beq(&mut self, instruction: &Instruction) {
        if self.status(StatusBit::Zero) {
            self.branch(instruction);
        }
    }
    
    pub fn bit(&mut self, instruction: &Instruction) {
        let value = self.resolve_operand(instruction);
        let ac = self.ac;
        self.set_status(StatusBit::Zero, ac & value == 0);
        self.set_status(StatusBit::Overflow, value & (1 << 6) != 0);
        self.set_status(StatusBit::Negative, value & (1 << 7) != 0);
    }
    
    pub fn bmi(&mut self, instruction: &Instruction) {
        if self.status(StatusBit::Negative) {
            self.branch(instruction);
        }
    }
    
    pub fn bne(&mut self, instruction: &Instruction) {
        if !self.status(StatusBit::Zero) {
            self.branch(instruction);
        }
    }
    
    pub fn bpl(&mut self, instruction: &Instruction) {
        if !self.status(StatusBit::Negative) {
            self.branch(instruction);
        }
    }
    
    pub fn brk(&mut self, _: &Instruction) {
        println!("BRK: {:?}", self);
    }
    
    pub fn clc(&mut self, _: &Instruction) {
        self.set_status(StatusBit::Carry, false);
    }
    
    pub fn cld(&mut self, _: &Instruction) {
        self.set_status(StatusBit::Decimal, false);
    }
    
    pub fn cli(&mut self, _: &Instruction) {
        self.set_status(StatusBit::Interrupt, true);
    }
    
    pub fn cmp(&mut self, instruction: &Instruction) {
        let value = self.resolve_operand(instruction);
        let ac = self.ac;
        self.set_status(StatusBit::Carry, ac >= value);
        self.update_status(ac.overflowing_sub(value).0);
    }
    
    pub fn cpx(&mut self, instruction: &Instruction) {
        let value = self.resolve_operand(instruction);
        let x = self.x;
        self.set_status(StatusBit::Carry, x >= value);
        self.update_status(x.overflowing_sub(value).0);
    }
    
    pub fn cpy(&mut self, instruction: &Instruction) {
        let value = self.resolve_operand(instruction);
        let y = self.y;
        self.set_status(StatusBit::Carry, y >= value);
        self.update_status(y.overflowing_sub(value).0);
    }
    
    pub fn dec(&mut self, instruction: &Instruction) {
        let addr = self.resolve_address(instruction);
        let value = self.bus.read(addr).overflowing_sub(1).0;
        self.bus.write(addr, value);
        self.update_status(value);
    }
    
    pub fn dex(&mut self, instruction: &Instruction) {
        self.x -= 1;
        self.update_x();
    }
    
    pub fn dey(&mut self, instruction: &Instruction) {
        self.x -= 1;
        self.update_y();
    }
    
    
    
    pub fn jmp(&mut self, instruction: &Instruction) {
        self.pc = self.resolve_address(instruction);
    }
}
