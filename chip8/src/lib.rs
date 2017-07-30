// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate collections;

extern crate rand;
extern crate byteorder;
#[macro_use]
extern crate enum_primitive;

#[cfg(feature = "std")]
extern crate core;

mod stack;
mod timer;
mod vm;
mod regfile;

pub mod display;
pub mod instruction;

pub use self::vm::Vm;
pub use self::vm::Env;

#[derive(Debug)]
pub enum Error {
    UnrecognizedInstruction(instruction::InstructionWord),
}

pub type Result<T> = core::result::Result<T, Error>;

#[cfg(feature = "std")]
impl std::error::Error for Error {
    /// A short description of the error.
    fn description(&self) -> &str {
        match *self {
            Error::UnrecognizedInstruction(_) => "unrecognized instruction",
        }
    }

    /// The lower level cause of this error, if any.
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
