use std::path::Path;
use std::io::Read;
use std::fs::File;
use std::env;

fn read_bin<P: AsRef<Path>>(path: P) -> Box<[u8]> {
    let mut bin_file = File::open(path).unwrap();
    let mut bin_buffer = Vec::new();
    bin_file.read_to_end(&mut bin_buffer);
    bin_buffer.into_boxed_slice()
}

fn main() {
    let bin_file_name = env::args().nth(1).unwrap();
    let bin_data = read_bin(bin_file_name); 
    
    println!("{:?}", bin_data);
    
    let mut chip8 = Chip8::new();
    chip8.execute(bin_data);
}

struct Chip8 {
    memory: [u8; 4096],
    gpr: [u8; 16]
    // TODO: PC register
    // TODO: I register
}

impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            memory: [0; 4096], // TODO: beware this stuff is going to be allocated on the stack
            gpr: [0; 16]
        }
    }
    
    fn execute(&mut self, bin_data: Box<[u8]>) {
        let first_byte = (*bin_data)[0] as u16;
        let second_byte = (*bin_data)[1] as u16;
        
        let instruction = (first_byte << 8) | second_byte; 
        
        panic!("instruction: {:#x}", instruction);
    }
    
    fn read_gpr(&self, index: u8) -> u8 {
        self.gpr[index as usize]
    }
}