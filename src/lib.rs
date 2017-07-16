#[macro_use]
extern crate enum_primitive;
extern crate num;

pub mod decode;
pub mod emu;

#[derive(Clone, Debug)]
pub enum Error {
    BadOpcode,
    BadFunct,
    BadRegister,
    MemoryOutOfBounds,
}

pub type Result<T> = std::result::Result<T, Error>;
