use num::FromPrimitive;

use {Error, Result};
use super::{Reg, Funct, Imm};

/// Halfway-decoded instruction.
///
/// This is decoded as far as the instruction format,
/// but not the particular opcode.
#[derive(Clone, Debug)]
pub struct Instruction<T> {
    pub opcode: Opcode,
    pub funct: Funct,
    pub operands: T,
}

enum_from_primitive! {
    /// The "opcode" field of the instruction encoding.
    ///
    /// This doesn't completely describe what an instruction does; for that
    /// you need the 'funct' field as well.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Opcode {
        Load    = 0b00_000_11,
     // MiscMem = 0b00_011_11,
        OpImm   = 0b00_100_11,
        Auipc   = 0b00_101_11,
     // OpImm32 = 0b00_110_11,
        Store   = 0b01_000_11,
        Op      = 0b01_100_11,
        Lui     = 0b01_101_11,
     // Op32    = 0b01_110_11,
        Branch  = 0b11_000_11,
        Jalr    = 0b11_001_11,
        Jal     = 0b11_011_11,
        System  = 0b11_100_11,
    }
}

impl Opcode {
    pub fn from_inst(bits: u32) -> Result<Opcode> {
        match FromPrimitive::from_u32(bits & 0b1111111) {
            Some(o) => Ok(o),
            None => Err(Error::BadOpcode),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ROperands {
    pub rd: Reg,
    pub rs1: Reg,
    pub rs2: Reg,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IOperands {
    pub rd: Reg,
    pub rs1: Reg,
    pub imm: Imm,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SOperands {
    pub rs1: Reg,
    pub rs2: Reg,
    pub imm: Imm,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BOperands {
    pub rs1: Reg,
    pub rs2: Reg,
    pub imm: Imm,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UOperands {
    pub rd: Reg,
    pub imm: Imm,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JOperands {
    pub rd: Reg,
    pub imm: Imm,
}

pub type RInstruction = Instruction<ROperands>;
pub type IInstruction = Instruction<IOperands>;
pub type SInstruction = Instruction<SOperands>;
pub type BInstruction = Instruction<BOperands>;
pub type UInstruction = Instruction<UOperands>;
pub type JInstruction = Instruction<JOperands>;

fn inst<T>(bits: u32, funct: Funct, operands: T) -> Result<Instruction<T>> {
    Ok(Instruction {
        opcode: Opcode::from_inst(bits)?,
        funct: funct,
        operands: operands,
    })
}

fn reg(bits: u32) -> Result<Reg> {
    Reg::new(bits & 0b11111)
}

fn funct3(bits: u32) -> Funct {
    ((bits >> 12) & 0b111) as Funct
}

fn funct7(bits: u32) -> Funct {
    ((bits >> 22) & 0b1111111_000) as Funct
}

// go from
//   00..00snn..nn
// to
//   ss..sssnn..nn
fn sign_extend(mut bits: u32, num_bits: u8) -> u32 {
    assert!(num_bits > 0);
    if 0 != bits & (1 << (num_bits - 1)) {
        for i in num_bits..32 {
            bits |= 1 << i;
        }
    }
    bits
}

pub fn decode_r(bits: u32) -> Result<RInstruction> {
    inst(bits, funct3(bits) | funct7(bits), ROperands {
        rd: reg(bits >> 7)?,
        rs1: reg(bits >> 15)?,
        rs2: reg(bits >> 20)?,
    })
}

pub fn decode_i(bits: u32) -> Result<IInstruction> {
    inst(bits, funct3(bits), IOperands {
        rd: reg(bits >> 7)?,
        rs1: reg(bits >> 15)?,
        imm: sign_extend(bits >> 20, 12),
    })
}

pub fn decode_s(bits: u32) -> Result<SInstruction> {
    inst(bits, funct3(bits), SOperands {
        rs1: reg(bits >> 15)?,
        rs2: reg(bits >> 20)?,
        imm: sign_extend(((bits >> 7) & 0b11111)
                       | ((bits >> 20) & 0b1111111_00000),
                       12),
    })
}

pub fn decode_b(bits: u32) -> Result<BInstruction> {
    inst(bits, funct3(bits), BOperands {
        rs1: reg(bits >> 15)?,
        rs2: reg(bits >> 20)?,
        imm: sign_extend(((bits >> 7) & 0b1111_0)
                       | ((bits >> 20) & 0b111111_0000_0)
                       | ((bits << 4) & 0b1_000000_0000_0)
                       | ((bits >> 19) & 0b1_0_000000_0000_0),
                       13),
    })
}

pub fn decode_u(bits: u32) -> Result<UInstruction> {
    inst(bits, 0, UOperands {
        rd: reg(bits >> 7)?,
        imm: bits & !0b111111111111,
    })
}

pub fn decode_j(bits: u32) -> Result<JInstruction> {
    inst(bits, 0, JOperands {
        rd: reg(bits >> 7)?,
        imm: sign_extend((bits & 0b11111111_000000000000)
                       | ((bits >> 9) & 0b100000000000)
                       | ((bits >> 20) & 0b11111111110)
                       | ((bits >> 11) & 0b1_00000000000000000000),
                       21),
    })
}

#[cfg(test)]
mod tests {
    use super::sign_extend;

    #[test]
    fn test_sign_extend() {
        assert_eq!(0, sign_extend(0, 5));
        assert_eq!(0b00011, sign_extend(0b00011, 5));
        assert_eq!(!0b1111, sign_extend(0b10000, 5));
        assert_eq!(!0b1100, sign_extend(0b10011, 5));
    }
}
