#[macro_use]
extern crate enum_primitive;
extern crate num;

pub mod decode;

#[derive(Clone, Debug)]
pub enum Error {
    BadOpcode,
    BadRegister,
}

pub type Result<T> = std::result::Result<T, Error>;
