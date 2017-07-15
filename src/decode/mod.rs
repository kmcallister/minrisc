use {Error, Result};

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
