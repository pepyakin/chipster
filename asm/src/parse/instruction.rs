use vm::instruction::Reg;

use parse::Statement;
use parse::Operand;
use parse::horizontal_spaces;
use parse::ident;

use combine::{Parser, ParserExt, parser};
use combine::primitives::{State, Stream, ParseResult};

pub fn instruction<I>(input: State<I>) -> ParseResult<Statement, I>
    where I: Stream<Item = char>
{
    use std::ascii::AsciiExt;
    use combine::{many1, letter, char, sep_by, parser, optional, between};

    let mnemonic =
        many1(letter()).map(|x: String| x.to_ascii_uppercase()).message("mnemonic expected");

    let operands = sep_by(between(horizontal_spaces(),
                                  horizontal_spaces(),
                                  parser(operand::<I>)),
                          char(','));

    let opt_operands = optional(between(horizontal_spaces(), horizontal_spaces(), operands));

    mnemonic.and(opt_operands)
        .map(|x| Statement::Instruction(x.0, x.1.unwrap_or_default()))
        .parse_state(input)
}

fn operand<I>(input: State<I>) -> ParseResult<Operand, I>
    where I: Stream<Item = char>
{
    use combine::{many1, digit, token, hex_digit, string};

    let literal = parser(|input| {
        let hex_literal = token('#')
            .with(many1(digit()))
            .and_then(|s: String| u16::from_str_radix(s.as_str(), 16))
            .map(Operand::new_literal);
        let dec_literal = many1(digit())
            .and_then(|s: String| s.parse::<u16>())
            .map(Operand::new_literal);

        hex_literal.or(dec_literal).parse_state(input)
    });
    let register = parser(|input| {
        token('V')
            .with(hex_digit())
            .map(|x: char| {
                let index = x.to_digit(16).unwrap() as u8;
                Reg::from_index(index)
            })
            .map(Operand::Register)
            .parse_state(input)
    });


    let index_reg = parser(|input| token('I').map(|_| Operand::IndexReg).parse_state(input));
    let deref_index_reg =
        parser(|input| string("[I]").map(|_| Operand::DerefIndexReg).parse_state(input));
    let font_designator = parser(|input| token('F').map(|_| Operand::F).parse_state(input));
    let bcd_designator = parser(|input| token('B').map(|_| Operand::B).parse_state(input));
    let kbd_designator = parser(|input| token('K').map(|_| Operand::K).parse_state(input));
    let dt_reg = string("DT").map(|_| Operand::DT);
    let st_reg = string("ST").map(|_| Operand::ST);
    let label = ident().map(Operand::new_label);

    literal.or(index_reg)
        .or(deref_index_reg)
        .or(font_designator)
        .or(bcd_designator)
        .or(kbd_designator)
        .or(dt_reg)
        .or(st_reg)
        .or(register)
        .or(label)
        .parse_state(input)
}

#[test]
fn test_instruction_no_operands() {
    let result = parser(instruction).parse("CLS");
    let expected = Statement::Instruction("CLS".to_string(), vec![]);
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_instruction_jumping_case() {
    let result = parser(instruction).parse("cLs");
    let expected = Statement::Instruction("CLS".to_string(), vec![]);
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_instruction_with_operand() {
    let result = parser(instruction).parse("SYS 228");
    let expected = Statement::Instruction("SYS".to_string(), vec![Operand::new_literal(228)]);
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_instruction_with_operand_tab_spaced() {
    let result = parser(instruction).parse("SYS\t228");
    let expected = Statement::Instruction("SYS".to_string(), vec![Operand::new_literal(228)]);
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_instruction_with_two_operands() {
    let result = parser(instruction).parse("SE Vf, 30");
    let expected = Statement::Instruction("SE".to_string(),
                                          vec![Operand::Register(Reg::Vf),
                                               Operand::new_literal(30)]);
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_instruction_with_spaced_operands_after_comma() {
    let result = parser(instruction).parse("DRW     V0,\tVa,15");
    let expected = Statement::Instruction("DRW".to_string(),
                                          vec![Operand::Register(Reg::V0),
                                               Operand::Register(Reg::Va),
                                               Operand::new_literal(15)]);
    assert_eq!(result, Ok((expected, "")))
}

#[test]
fn test_instruction_with_spaced_operands_before_comma() {
    let result = parser(instruction).parse("DRW     V0 ,Va\t,15");
    let expected = Statement::Instruction("DRW".to_string(),
                                          vec![Operand::Register(Reg::V0),
                                               Operand::Register(Reg::Va),
                                               Operand::new_literal(15)]);
    assert_eq!(result, Ok((expected, "")))
}

#[test]
fn test_operand_dec_literal() {
    let result = parser(operand).parse("1312");
    let expected = Operand::new_literal(1312);

    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_operand_hex_literal() {
    let result = parser(operand).parse("#228");
    let expected = Operand::new_literal(0x228);

    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_operand_index_reg() {
    let result = parser(operand).parse("I");
    let expected = Operand::IndexReg;

    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_operand_gpr() {
    let result = parser(operand).parse("V9");
    let expected = Operand::Register(Reg::V9);

    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_operand_deref_index_reg() {
    let result = parser(operand).parse("[I]");
    let expected = Operand::DerefIndexReg;

    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_operand_f() {
    let result = parser(operand).parse("F");
    let expected = Operand::F;

    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_operand_b() {
    let result = parser(operand).parse("B");
    let expected = Operand::B;

    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_operand_k() {
    let result = parser(operand).parse("K");
    let expected = Operand::K;

    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_operand_st() {
    let result = parser(operand).parse("DT");
    let expected = Operand::DT;

    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_operand_dt() {
    let result = parser(operand).parse("ST");
    let expected = Operand::ST;

    assert_eq!(result, Ok((expected, "")));
}
