
use super::parse::*;

use vm::instruction::Addr;
use vm::instruction::Imm;
use vm::instruction::Instruction;
use super::vm;

impl LiteralValue {
    fn as_imm(&self) -> Imm {
        Imm(self.as_u8())
    }

    fn as_addr(&self) -> Addr {
        Addr(self.as_u16())
    }
}

pub fn translate(statements: Vec<Statement>) -> Box<[u8]> {
    let mut instructions: Vec<vm::instruction::Instruction> = vec![];
    for statement in statements.into_iter() {
        match statement {
            Statement::Instruction(mnemonic, operands) => {
                println!("mnemonic={:?} operands={:?}", mnemonic, operands);
                let instruction = match_instruction(&mnemonic, operands);
                instructions.push(instruction);
            }
            _ => {}
        }
    }

    encode_instructions(instructions)
}

fn match_instruction(mnemonic: &str, operands: Vec<Operand>) -> vm::instruction::Instruction {
    let unsupported_operands =
        || format!("unsupported operands {:?} for {:?}", &operands, mnemonic);

    match mnemonic {
        "CLS" => vm::instruction::Instruction::ClearScreen,
        "JUMP" => {
            match &operands[..] {
                [Operand::Literal(lit)] => Instruction::Jump(lit.as_addr()),
                _ => panic!(unsupported_operands()),
            }
        }
        "CALL" => {
            match &operands[..] {
                [Operand::Literal(lit)] => Instruction::Call(lit.as_addr()),
                _ => panic!(unsupported_operands()),
            }
        }
        "SE" => {
            match &operands[..] {
                [Operand::Register(vx), Operand::Literal(kk)] => {
                    Instruction::SkipEqImm {
                        vx: vx,
                        imm: kk.as_imm(),
                        inv: false,
                    }
                }
                [Operand::Register(vx), Operand::Register(vy)] => {
                    Instruction::SkipEqReg {
                        vx: vx,
                        vy: vy,
                        inv: false,
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        "SNE" => {
            match &operands[..] {
                [Operand::Register(vx), Operand::Literal(kk)] => {
                    Instruction::SkipEqImm {
                        vx: vx,
                        imm: kk.as_imm(),
                        inv: true,
                    }
                }
                [Operand::Register(vx), Operand::Register(vy)] => {
                    Instruction::SkipEqReg {
                        vx: vx,
                        vy: vy,
                        inv: true,
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        _ => unimplemented!(),
    }
}

fn encode_instructions(instructions: Vec<Instruction>) -> Box<[u8]> {
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
