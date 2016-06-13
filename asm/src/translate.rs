
use parse::*;

use vm::instruction::Addr;
use vm::instruction::Reg;
use vm::instruction::Imm;
use vm::instruction::Imm4;
use vm::instruction::Instruction;
use vm::instruction::Fun;
use super::vm;

use std::collections::HashMap;

impl LiteralValue {
    fn as_imm(&self) -> Imm {
        Imm(self.as_u8())
    }

    fn as_imm4(&self) -> Imm4 {
        Imm4(self.as_u4())
    }

    fn as_addr(&self) -> Addr {
        Addr(self.as_u16())
    }
}

pub fn translate(statements: Vec<Statement>) -> Box<[u8]> {
    let mut instruction_statements: Vec<(String, Vec<Operand>)> =
        Vec::with_capacity(statements.len());
    let mut label_map = HashMap::new();

    for statement in statements.into_iter() {
        match statement {
            Statement::Instruction(mnemonic, operands) => {
                instruction_statements.push((mnemonic, operands));
            }
            Statement::Label(label) => {
                // Based on assumption that all instructions use two bytes.
                let addr = Addr(0x200 + (instruction_statements.len() * 2) as u16);

                if !label_map.contains_key(&label) {
                    label_map.insert(label, addr);
                } else {
                    panic!("duplicate label: {}", label)
                }
            }
        }
    }

    let instructions = instruction_statements.into_iter()
        .map(|(mnemonic, operands)| match_instruction(&mnemonic, &operands, &label_map))
        .collect::<Vec<_>>();

    emit_instructions(instructions)
}

fn match_instruction(mnemonic: &str,
                     operands: &Vec<Operand>,
                     label_map: &HashMap<String, Addr>)
                     -> Instruction {
    let resolve_label = |label: &String| *label_map.get(label).expect("Undefined label");
    let unsupported_operands =
        || format!("unsupported operands {:?} for {:?}", &operands, mnemonic);

    match mnemonic {
        "CLS" => vm::instruction::Instruction::ClearScreen,
        "RET" => Instruction::Ret,
        "SYS" => {
            match &operands[..] {
                &[Operand::Literal(lit)] => Instruction::Sys(lit.as_addr()),
                &[Operand::Label(ref label)] => Instruction::Sys(resolve_label(label)),
                _ => panic!(unsupported_operands()),
            }
        }
        "JP" => {
            match &operands[..] {
                &[Operand::Literal(lit)] => Instruction::Jump(lit.as_addr()),
                &[Operand::Label(ref label)] => Instruction::Jump(resolve_label(label)),
                &[Operand::Register(Reg::V0), Operand::Literal(lit)] => {
                    Instruction::JumpPlusV0(lit.as_addr())
                }
                &[Operand::Register(Reg::V0), Operand::Label(ref label)] => {
                    Instruction::JumpPlusV0(resolve_label(label))
                } 
                _ => panic!(unsupported_operands()),
            }
        }
        "CALL" => {
            match &operands[..] {
                &[Operand::Literal(lit)] => Instruction::Call(lit.as_addr()),
                &[Operand::Label(ref label)] => Instruction::Call(resolve_label(label)),
                _ => panic!(unsupported_operands()),
            }
        }
        "SE" => {
            match &operands[..] {
                &[Operand::Register(vx), Operand::Literal(kk)] => {
                    Instruction::SkipEqImm {
                        vx: vx,
                        imm: kk.as_imm(),
                        inv: false,
                    }
                }
                &[Operand::Register(vx), Operand::Register(vy)] => {
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
                &[Operand::Register(vx), Operand::Literal(kk)] => {
                    Instruction::SkipEqImm {
                        vx: vx,
                        imm: kk.as_imm(),
                        inv: true,
                    }
                }
                &[Operand::Register(vx), Operand::Register(vy)] => {
                    Instruction::SkipEqReg {
                        vx: vx,
                        vy: vy,
                        inv: true,
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        "LD" => {
            match &operands[..] {
                &[Operand::Register(vx), Operand::Literal(kk)] => {
                    Instruction::PutImm {
                        vx: vx,
                        imm: kk.as_imm(),
                    }
                }
                &[Operand::Register(vx), Operand::Register(vy)] => {
                    Instruction::Apply {
                        vx: vx,
                        vy: vy,
                        f: Fun::Id,
                    }
                }
                &[Operand::IndexReg, Operand::Literal(kk)] => Instruction::SetI(kk.as_addr()),
                &[Operand::IndexReg, Operand::Label(ref label)] => {
                    Instruction::SetI(resolve_label(label))
                }
                &[Operand::Register(vx), Operand::DT] => Instruction::GetDT(vx),
                &[Operand::Register(vx), Operand::K] => Instruction::WaitKey(vx),
                &[Operand::DT, Operand::Register(vx)] => Instruction::SetDT(vx),
                &[Operand::ST, Operand::Register(vx)] => Instruction::SetST(vx),
                &[Operand::F, Operand::Register(vx)] => Instruction::LoadGlyph(vx),
                &[Operand::B, Operand::Register(vx)] => Instruction::StoreBCD(vx),
                &[Operand::DerefIndexReg, Operand::Register(vx)] => Instruction::StoreRegs(vx),
                &[Operand::Register(vx), Operand::DerefIndexReg] => Instruction::LoadRegs(vx),

                _ => panic!(unsupported_operands()),
            }
        }
        "ADD" => {
            match &operands[..] {
                &[Operand::Register(vx), Operand::Literal(kk)] => {
                    Instruction::AddImm {
                        vx: vx,
                        imm: kk.as_imm(),
                    }
                }
                &[Operand::IndexReg, Operand::Register(vx)] => Instruction::AddI(vx),
                &[Operand::Register(vx), Operand::Register(vy)] => {
                    Instruction::Apply {
                        vx: vx,
                        vy: vy,
                        f: Fun::Add,
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        "OR" => {
            match &operands[..] {
                &[Operand::Register(vx), Operand::Register(vy)] => {
                    Instruction::Apply {
                        vx: vx,
                        vy: vy,
                        f: Fun::Or,
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        "AND" => {
            match &operands[..] {
                &[Operand::Register(vx), Operand::Register(vy)] => {
                    Instruction::Apply {
                        vx: vx,
                        vy: vy,
                        f: Fun::And,
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        "XOR" => {
            match &operands[..] {
                &[Operand::Register(vx), Operand::Register(vy)] => {
                    Instruction::Apply {
                        vx: vx,
                        vy: vy,
                        f: Fun::Xor,
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        "SUB" => {
            match &operands[..] {
                &[Operand::Register(vx), Operand::Register(vy)] => {
                    Instruction::Apply {
                        vx: vx,
                        vy: vy,
                        f: Fun::Subtract,
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        "SHR" => {
            match &operands[..] {
                &[Operand::Register(vx), Operand::Register(vy)] => {
                    Instruction::Apply {
                        vx: vx,
                        vy: vy,
                        f: Fun::ShiftRight,
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        "SHL" => {
            match &operands[..] {
                &[Operand::Register(vx), Operand::Register(vy)] => {
                    Instruction::Apply {
                        vx: vx,
                        vy: vy,
                        f: Fun::ShiftLeft,
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        "SUBN" => {
            match &operands[..] {
                &[Operand::Register(vx), Operand::Register(vy)] => {
                    Instruction::Apply {
                        vx: vx,
                        vy: vy,
                        f: Fun::SubtractInv,
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        "SKP" => {
            match &operands[..] {
                &[Operand::Register(vx)] => {
                    Instruction::SkipPressed {
                        vx: vx,
                        inv: false,
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        "SKNP" => {
            match &operands[..] {
                &[Operand::Register(vx)] => {
                    Instruction::SkipPressed {
                        vx: vx,
                        inv: true,
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        "DRW" => {
            match &operands[..] {
                &[Operand::Register(vx), Operand::Register(vy), Operand::Literal(lit)] => {
                    Instruction::Draw {
                        vx: vx,
                        vy: vy,
                        n: lit.as_imm4(),
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        "RND" => {
            match &operands[..] {
                &[Operand::Register(vx), Operand::Literal(lit)] => {
                    Instruction::Randomize {
                        vx: vx,
                        imm: lit.as_imm(),
                    }
                }
                _ => panic!(unsupported_operands()),
            }
        }
        _ => unimplemented!(),
    }
}

fn emit_instructions(instructions: Vec<Instruction>) -> Box<[u8]> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use parse::*;

    #[test]
    fn test_compile_label_forwad() {
        let statements: Vec<Statement> = vec![Statement::Instruction("CALL".to_string(),
                                                                     vec![Operand::Label("foo"
                                                                              .to_string())]),
                                              Statement::Label("foo".to_string()),
                                              Statement::Instruction("RET".to_string(), vec![])];
        let binary = translate(statements);

        assert_eq!(binary, vec![0x22, 0x02, 0x00, 0xEE].into_boxed_slice());
    }
}
