extern crate combine;
extern crate vm;

mod parse;

use vm::instruction::Reg;
use vm::instruction::Addr;

use parse::Operand;
use parse::Statement;

pub fn compile(source: &str) -> Box<[u8]> {
    let statements = parse::parse_source(source);
    
    let mut instructions: Vec<vm::instruction::Instruction> = vec![];

    for statement in statements {
        match statement {
            Statement::Instruction(mnemonic, operands) => {
                println!("mnemonic={:?} operands={:?}", mnemonic, operands);
                let instruction = match mnemonic.as_ref() {
                    "CLS" => vm::instruction::Instruction::ClearScreen,
                    "CALL" => {
                        let t = operands.iter().nth(0).unwrap();
                        match *t {
                            Operand::Literal(lit) => vm::instruction::Instruction::Call(Addr(lit)),
                            _ => panic!("unsupported operand for CALL"),
                        }
                    }
                    _ => unimplemented!(),
                };
                instructions.push(instruction);
            }
            _ => {}
        }
    }
   
    instructions.into_iter()
        .flat_map::<Vec<u8>, _>(|instruction| {
            fn unpack_word(word: u16) -> Vec<u8> {
                // TODO: byteorder
                let first_byte = ((word >> 8) & 0xFF) as u8;
                let second_byte = (word & 0xFF) as u8;
                
                vec![first_byte, second_byte]
            }
            
            let instruction_word = instruction.encode();
            unpack_word(instruction_word.0)
        })
        .collect::<Vec<u8>>()
        .into_boxed_slice()
}

#[test]
fn compile_simple_instruction() {
    let compiled: Box<[u8]> = compile("CALL 32");
    let expected: Box<[u8]> = vec![0x20, 0x20].into_boxed_slice();

    assert_eq!(compiled, expected);
}
