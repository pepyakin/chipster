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
    let bin_file_name = env::args().nth(0).unwrap();
    let bin_data = read_bin(bin_file_name); 
    
    
}
