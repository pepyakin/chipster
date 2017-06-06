// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate rand;
extern crate byteorder;
#[macro_use]
extern crate enum_primitive;
#[macro_use]
extern crate error_chain;

mod stack;
mod timer;
mod vm;
mod regfile;

pub mod display;
pub mod instruction;

pub use self::vm::Vm;
pub use self::vm::Env;

error_chain! {
    links {
        Instruction(instruction::Error, instruction::ErrorKind);
    }
}
