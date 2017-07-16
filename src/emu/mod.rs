use std::io;

use decode;
use decode::Reg;
use decode::Instruction::*;
use decode::formats::{IOperands, ROperands, BOperands};
use {Error, Result};

#[derive(Clone)]
pub struct Machine {
    pub pc: u32,
    iregs: [u32; 31],
    pub memory: Vec<u8>,
}

#[derive(Clone, Debug)]
pub enum StepOutcome {
    Running,
    Syscall,
    Breakpoint,
}

impl Machine {
    pub fn dump<W>(&self, writer: &mut W)
        where W: io::Write,
    {
        write!(writer, "PC : {:08X}\n", self.pc).unwrap();
        for i in 0..32 {
            let v = self.get_reg(Reg::new(i).unwrap());
            write!(writer, "R{:<2}: {:08X}    ", i, v).unwrap();

            if (i % 4) == 3 {
                write!(writer, "\n").unwrap();
            }
        }
    }

    pub fn with_memory(size: usize) -> Machine {
        Machine {
            pc: 0,
            iregs: [0; 31],
            memory: vec![0; size],
        }
    }

    pub fn load8(&self, addr: u32) -> Result<u8> {
        if addr as usize >= self.memory.len() {
            return Err(Error::MemoryOutOfBounds);
        }

        Ok(self.memory[addr as usize])
    }

    pub fn load16(&self, addr: u32) -> Result<u16> {
        if let Some(last_addr) = addr.checked_add(1) {
            if (last_addr as usize) < self.memory.len() {
                return Ok(
                    (self.memory[addr as usize] as u16)
                    | ((self.memory[(addr+1) as usize] as u16) << 8));
            }
        }

        Err(Error::MemoryOutOfBounds)
    }

    pub fn load32(&self, addr: u32) -> Result<u32> {
        if let Some(last_addr) = addr.checked_add(3) {
            if (last_addr as usize) < self.memory.len() {
                return Ok(
                    (self.memory[addr as usize] as u32)
                    | ((self.memory[(addr+1) as usize] as u32) << 8)
                    | ((self.memory[(addr+2) as usize] as u32) << 16)
                    | ((self.memory[(addr+3) as usize] as u32) << 24));
            }
        }

        Err(Error::MemoryOutOfBounds)
    }

    pub fn store8(&mut self, addr: u32, val: u8) -> Result<()> {
        if addr as usize >= self.memory.len() {
            return Err(Error::MemoryOutOfBounds);
        }

        self.memory[addr as usize] = val;
        Ok(())
    }

    pub fn store16(&mut self, addr: u32, val: u16) -> Result<()> {
        if let Some(last_addr) = addr.checked_add(1) {
            if (last_addr as usize) < self.memory.len() {
                self.memory[addr as usize] = (val & 0xFF) as u8;
                self.memory[(addr+1) as usize] = ((val >> 8) & 0xFF) as u8;
                return Ok(());
            }
        }

        Err(Error::MemoryOutOfBounds)
    }

    pub fn store32(&mut self, addr: u32, val: u32) -> Result<()> {
        if let Some(last_addr) = addr.checked_add(3) {
            if (last_addr as usize) < self.memory.len() {
                self.memory[addr as usize] = (val & 0xFF) as u8;
                self.memory[(addr+1) as usize] = ((val >> 8) & 0xFF) as u8;
                self.memory[(addr+2) as usize] = ((val >> 16) & 0xFF) as u8;
                self.memory[(addr+3) as usize] = ((val >> 24) & 0xFF) as u8;
                return Ok(());
            }
        }

        Err(Error::MemoryOutOfBounds)
    }

    pub fn get_reg(&self, reg: Reg) -> u32 {
        match reg.num() as usize {
            0 => 0,
            n => self.iregs[n - 1],
        }
    }

    pub fn set_reg(&mut self, reg: Reg, val: u32) {
        let num = reg.num() as usize;
        if num > 0 {
            self.iregs[num - 1] = val;
        }
    }

    fn op_imm<F>(&mut self, op: &IOperands, f: F)
        where F: FnOnce(u32, u32) -> u32,
    {
        let res = f(self.get_reg(op.rs1), op.imm);
        self.set_reg(op.rd, res);
    }

    fn op_reg<F>(&mut self, op: &ROperands, f: F)
        where F: FnOnce(u32, u32) -> u32,
    {
        let res = f(self.get_reg(op.rs1), self.get_reg(op.rs2));
        self.set_reg(op.rd, res);
    }

    fn branch<C>(&mut self, op: &BOperands, next_pc: &mut u32, cond: C)
        where C: FnOnce(u32, u32) -> bool,
    {
        if cond(self.get_reg(op.rs1), self.get_reg(op.rs2)) {
            *next_pc = self.pc.wrapping_add(op.imm);
        }
    }

    pub fn step(&mut self) -> Result<StepOutcome> {
        let mut next_pc = self.pc.wrapping_add(4);
        let mut outcome = StepOutcome::Running;

        match decode::decode(self.load32(self.pc)?)? {
            ADDI(ref op) => self.op_imm(op, |x, y| x.wrapping_add(y)),
            ANDI(ref op) => self.op_imm(op, |x, y| x & y),
             ORI(ref op) => self.op_imm(op, |x, y| x | y),
            XORI(ref op) => self.op_imm(op, |x, y| x ^ y),
            SLLI(ref op) => self.op_imm(op, |x, y| x << (y & 0b_11111)),
            SRLI(ref op) => self.op_imm(op, |x, y| x >> (y & 0b_11111)),
            SRAI(ref op) => self.op_imm(op, |x, y| ((x as i32) >> (y & 0b_11111)) as u32),

            SLTI(ref op) => self.op_imm(op, |x, y| {
                if (x as i32) < (y as i32) { 1 } else { 0 }
            }),

            SLTIU(ref op) => self.op_imm(op, |x, y| {
                if x < y { 1 } else { 0 }
            }),

            ADD(ref op) => self.op_reg(op, |x, y| x.wrapping_add(y)),
            SUB(ref op) => self.op_reg(op, |x, y| x.wrapping_sub(y)),
            AND(ref op) => self.op_reg(op, |x, y| x & y),
             OR(ref op) => self.op_reg(op, |x, y| x | y),
            XOR(ref op) => self.op_reg(op, |x, y| x ^ y),
            SLL(ref op) => self.op_reg(op, |x, y| x << y),
            SRL(ref op) => self.op_reg(op, |x, y| x >> y),
            SRA(ref op) => self.op_reg(op, |x, y| ((x as i32) >> y) as u32),

            SLT(ref op) => self.op_reg(op, |x, y| {
                if (x as i32) < (y as i32) { 1 } else { 0 }
            }),

            SLTU(ref op) => self.op_reg(op, |x, y| {
                if x < y { 1 } else { 0 }
            }),

            LUI(ref op) => self.set_reg(op.rd, op.imm),

            AUIPC(ref op) => {
                let res = op.imm.wrapping_add(self.pc);
                self.set_reg(op.rd, res);
            }

            JAL(ref op) => {
                self.set_reg(op.rd, next_pc);
                next_pc = self.pc.wrapping_add(op.imm);
            }

            JALR(ref op) => {
                self.set_reg(op.rd, next_pc);
                next_pc = self.get_reg(op.rs1).wrapping_add(op.imm) & !1;
            }

             BEQ(ref op) => self.branch(op, &mut next_pc, |x, y| x == y),
             BNE(ref op) => self.branch(op, &mut next_pc, |x, y| x != y),
             BLT(ref op) => self.branch(op, &mut next_pc, |x, y| (x as i32) < (y as i32)),
            BLTU(ref op) => self.branch(op, &mut next_pc, |x, y| x < y),
             BGE(ref op) => self.branch(op, &mut next_pc, |x, y| (x as i32) >= (y as i32)),
            BGEU(ref op) => self.branch(op, &mut next_pc, |x, y| x >= y),

            LW(ref op) => {
                let addr = self.get_reg(op.rs1).wrapping_add(op.imm);
                let val = self.load32(addr)?;
                self.set_reg(op.rd, val);
            }

            LH(ref op) => {
                let addr = self.get_reg(op.rs1).wrapping_add(op.imm);
                let val = self.load16(addr)? as i16;
                self.set_reg(op.rd, val as i32 as u32);
            }

            LHU(ref op) => {
                let addr = self.get_reg(op.rs1).wrapping_add(op.imm);
                let val = self.load16(addr)?;
                self.set_reg(op.rd, val as u32);
            }

            LB(ref op) => {
                let addr = self.get_reg(op.rs1).wrapping_add(op.imm);
                let val = self.load8(addr)? as i8;
                self.set_reg(op.rd, val as i32 as u32);
            }

            LBU(ref op) => {
                let addr = self.get_reg(op.rs1).wrapping_add(op.imm);
                let val = self.load8(addr)?;
                self.set_reg(op.rd, val as u32);
            }

            SW(ref op) => {
                let addr = self.get_reg(op.rs1).wrapping_add(op.imm);
                let val = self.get_reg(op.rs2);
                self.store32(addr, val)?;
            }

            SH(ref op) => {
                let addr = self.get_reg(op.rs1).wrapping_add(op.imm);
                let val = self.get_reg(op.rs2) & 0xFFFF;
                self.store16(addr, val as u16)?;
            }

            SB(ref op) => {
                let addr = self.get_reg(op.rs1).wrapping_add(op.imm);
                let val = self.get_reg(op.rs2) & 0xFF;
                self.store8(addr, val as u8)?;
            }

            ECALL => {
                outcome = StepOutcome::Syscall;
            }

            EBREAK => {
                outcome = StepOutcome::Breakpoint;
            }
        }

        self.pc = next_pc;

        Ok(outcome)
    }
}
