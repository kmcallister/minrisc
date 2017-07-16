use {Error, Result};
use self::formats::{ROperands, IOperands, SOperands, BOperands, UOperands, JOperands};

pub mod formats;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Reg(u8);

impl Reg {
    pub fn new(n: u32) -> Result<Reg> {
        if n < 32 {
            Ok(Reg(n as u8))
        } else {
            Err(Error::BadRegister)
        }
    }

    pub fn num(&self) -> u8 {
        self.0
    }
}

pub type Funct = u16;
pub type Imm   = u32;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    ADDI(IOperands),
    SLTI(IOperands),
    SLTIU(IOperands),
    ANDI(IOperands),
    ORI(IOperands),
    XORI(IOperands),
    SLLI(IOperands),
    SRLI(IOperands),
    SRAI(IOperands),

    LUI(UOperands),
    AUIPC(UOperands),

    ADD(ROperands),
    SLT(ROperands),
    SLTU(ROperands),
    AND(ROperands),
    OR(ROperands),
    XOR(ROperands),
    SLL(ROperands),
    SRL(ROperands),
    SRA(ROperands),
    SUB(ROperands),

    JAL(JOperands),
    JALR(IOperands),

    BEQ(BOperands),
    BNE(BOperands),
    BLT(BOperands),
    BLTU(BOperands),
    BGE(BOperands),
    BGEU(BOperands),

    LW(IOperands),
    LH(IOperands),
    LHU(IOperands),
    LB(IOperands),
    LBU(IOperands),
    SW(SOperands),
    SH(SOperands),
    SB(SOperands),

    ECALL,
    EBREAK,

    // Not implemented:
    //     FENCE FENCE.I
    //     CSRRW CSRRS CSRRC CSRRWI CSRRSI CSRRCI
    //     RDCYCLE RDCYCLEH RDTIME RDTIMEH RDINSTRET RDINSTRETH
}

macro_rules! instruction {
    ($opcode:ident, $inst:expr) => {
        Ok(Instruction::$opcode($inst.operands))
    };
}

pub fn decode(bits: u32) -> Result<Instruction> {
    match formats::Opcode::from_inst(bits)? {
        formats::Opcode::OpImm => {
            let inst = formats::decode_i(bits)?;
            match inst.funct {
                0b_000 => instruction!(ADDI,  inst),
                0b_010 => instruction!(SLTI,  inst),
                0b_011 => instruction!(SLTIU, inst),
                0b_100 => instruction!(XORI,  inst),
                0b_110 => instruction!(ORI,   inst),
                0b_111 => instruction!(ANDI,  inst),

                0b_001 if inst.operands.imm & 0b_1111111_00000 == 0b0000000_00000
                    => instruction!(SLLI, inst),
                0b_101 if inst.operands.imm & 0b_1111111_00000 == 0b0000000_00000
                    => instruction!(SRLI, inst),
                0b_101 if inst.operands.imm & 0b_1111111_00000 == 0b0100000_00000
                    => instruction!(SRAI, inst),

                _ => Err(Error::BadFunct),
            }
        }

        formats::Opcode::Op => {
            let inst = formats::decode_r(bits)?;
            match inst.funct {
                0b_000 => instruction!(ADD,  inst),
                0b_101 => instruction!(SRL,  inst),
                0b_001 => instruction!(SLL,  inst),
                0b_010 => instruction!(SLT,  inst),
                0b_011 => instruction!(SLTU, inst),
                0b_100 => instruction!(XOR,  inst),
                0b_110 => instruction!(OR,   inst),
                0b_111 => instruction!(AND,  inst),

                0b_0100000_000 => instruction!(SUB, inst),
                0b_0100000_101 => instruction!(SRA, inst),

                _ => Err(Error::BadFunct),
            }
        }

        formats::Opcode::Lui
            => instruction!(LUI, formats::decode_u(bits)?),

        formats::Opcode::Auipc
            => instruction!(AUIPC, formats::decode_u(bits)?),

        formats::Opcode::Jal
            => instruction!(JAL, formats::decode_j(bits)?),

        formats::Opcode::Jalr
            => instruction!(JALR, formats::decode_i(bits)?),

        formats::Opcode::Branch => {
            let inst = formats::decode_b(bits)?;
            match inst.funct {
                0b_000 => instruction!(BEQ,  inst),
                0b_001 => instruction!(BNE,  inst),
                0b_100 => instruction!(BLT,  inst),
                0b_101 => instruction!(BGE,  inst),
                0b_110 => instruction!(BLTU, inst),
                0b_111 => instruction!(BGEU, inst),

                _ => Err(Error::BadFunct),
            }
        }

        formats::Opcode::Load => {
            let inst = formats::decode_i(bits)?;
            match inst.funct {
                0b_000 => instruction!(LB,  inst),
                0b_001 => instruction!(LH,  inst),
                0b_010 => instruction!(LW,  inst),
                0b_100 => instruction!(LBU, inst),
                0b_101 => instruction!(LHU, inst),

                _ => Err(Error::BadFunct),
            }
        }

        formats::Opcode::Store => {
            let inst = formats::decode_s(bits)?;
            match inst.funct {
                0b_000 => instruction!(SB, inst),
                0b_001 => instruction!(SH, inst),
                0b_010 => instruction!(SW, inst),

                _ => Err(Error::BadFunct),
            }
        }

        formats::Opcode::System => {
            let inst = formats::decode_i(bits)?;
            match inst.funct {
                0b_000 if inst.operands.rs1.num() == 0
                         && inst.operands.rd.num() == 0 => {
                    match inst.operands.imm {
                        0 => Ok(Instruction::ECALL),
                        1 => Ok(Instruction::EBREAK),
                        _ => Err(Error::BadFunct),
                    }
                }

                _ => Err(Error::BadFunct),
            }
        }
    }
}
