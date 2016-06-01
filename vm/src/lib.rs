extern crate rand;
extern crate byteorder;
#[macro_use] 
extern crate enum_primitive;

mod stack;
mod timer;
mod display;
mod vm;
mod instruction;

pub use self::vm::Chip8;