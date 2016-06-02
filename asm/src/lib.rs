#![feature(slice_patterns)]

extern crate combine;
extern crate vm;

mod parse;

use vm::instruction::Addr;
use vm::instruction::Imm;

use parse::Operand;
use parse::Statement;
use parse::LiteralValue;

impl LiteralValue {
    fn as_imm(&self) -> Imm {
        Imm(self.as_u8())
    }

    fn as_addr(&self) -> Addr {
        Addr(self.as_u16())
    }
}

pub fn compile(source: &str) -> Box<[u8]> {
    let statements = parse::parse_source(source);

    let mut instructions: Vec<vm::instruction::Instruction> = vec![];
    for statement in statements.into_iter() {
        match statement {
            Statement::Instruction(mnemonic, operands) => {
                println!("mnemonic={:?} operands={:?}", mnemonic, operands);
                let instruction = match mnemonic.as_ref() {
                    "CLS" => vm::instruction::Instruction::ClearScreen,
                    "CALL" => {
                        match &operands[..] {
                            [Operand::Literal(lit)] => {
                                vm::instruction::Instruction::Call(lit.as_addr())
                            }
                            _ => panic!("unsupported operand for CALL"),
                        }
                    }
                    "SE" => {
                        match &operands[..] {
                            [Operand::Register(vx), Operand::Literal(kk)] => {
                                vm::instruction::Instruction::SkipEqImm {
                                    vx: vx,
                                    imm: kk.as_imm(),
                                    inv: false,
                                }
                            }
                            [Operand::Register(vx), Operand::Register(vy)] => {
                                vm::instruction::Instruction::SkipEqReg {
                                    vx: vx,
                                    vy: vy,
                                    inv: false,
                                }
                            }
                            _ => panic!("unsupported operands {:?}", operands),
                        }
                    }
                    "SNE" => {
                        match &operands[..] {
                            [Operand::Register(vx), Operand::Literal(kk)] => {
                                vm::instruction::Instruction::SkipEqImm {
                                    vx: vx,
                                    imm: kk.as_imm(),
                                    inv: true,
                                }
                            }
                            [Operand::Register(vx), Operand::Register(vy)] => {
                                vm::instruction::Instruction::SkipEqReg {
                                    vx: vx,
                                    vy: vy,
                                    inv: true,
                                }
                            }
                            _ => panic!("unsupported operands {:?}", operands),
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
fn compile_unary_instruction() {
    let compiled: Box<[u8]> = compile("CALL 32");
    let expected: Box<[u8]> = vec![0x20, 0x20].into_boxed_slice();

    assert_eq!(compiled, expected);
}

#[test]
fn compile_binary_instruction() {
    let compiled: Box<[u8]> = compile("SE V0, Va");
    let expected: Box<[u8]> = vec![0x50, 0xA0].into_boxed_slice();

    assert_eq!(compiled, expected);
}
