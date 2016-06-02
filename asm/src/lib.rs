extern crate combine;
extern crate vm;

use vm::instruction::Reg;

use combine::{Parser, ParserExt, parser};
use combine::primitives::{State, Stream, ParseResult};

#[derive(Debug, PartialEq)]
struct Label(String);

#[derive(Debug, PartialEq)]
enum Operand {
    Literal(u16),
    Label(String),
    Register(Reg),
}

#[derive(Debug, PartialEq)]
enum Statement {
    Label(String),
    Instruction(String, Vec<Operand>),
}

pub fn compile(source: &str) -> Box<[u8]> {
    unimplemented!();
}

fn stmt<I>(input: State<I>) -> ParseResult<Statement, I>
    where I: Stream<Item = char>
{
    unimplemented!();
}

fn label<I>(input: State<I>) -> ParseResult<Label, I>
    where I: Stream<Item = char>
{
    use combine::{many1, letter, token};
    
    let ident = many1(letter().or(token('_')));
    let label = ident.map(Label);

    label.skip(token(':')).parse_state(input)
}

fn instruction<I>(input: State<I>) -> ParseResult<Statement, I>
    where I: Stream<Item = char>
{
    use combine::{many1, letter, char, spaces, sep_by, parser};
    
    let mnemonic = many1(letter());

    let lex_char = |c| char(c).skip(spaces());
    let operands = sep_by(parser(operand::<I>), lex_char(','));

    mnemonic.skip(spaces())
        .and(operands)
        .map(|x| Statement::Instruction(x.0, x.1))
        .parse_state(input)
}

fn operand<I>(input: State<I>) -> ParseResult<Operand, I>
    where I: Stream<Item = char>
{
    use combine::{many1, digit, token, hex_digit};
    
    let literal = many1(digit()).and_then(|s: String| s.parse::<u16>()).map(Operand::Literal);
    let register = token('V')
        .with(hex_digit())
        .map(|x: char| {
            let index = x.to_digit(16).unwrap() as u8;
            Reg::from_index(index)
        })
        .map(Operand::Register);

    literal.or(register).parse_state(input)
}

#[test]
fn test_instruction_no_operands() {
    let result = parser(instruction).parse("CLS");
    let expected = Statement::Instruction("CLS".to_string(), vec![]);
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_instruction_with_operand() {
    let result = parser(instruction).parse("SYS 228");
    let expected = Statement::Instruction("SYS".to_string(), vec![Operand::Literal(228)]);
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_instruction_with_two_operands() {
    let result = parser(instruction).parse("SE Vf, 30");
    let expected = Statement::Instruction("SE".to_string(),
                                          vec![Operand::Register(Reg::Vf), Operand::Literal(30)]);
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_instruction_with_spaced_operands() {
    let result = parser(instruction).parse("DRW     V0,\tVa,15");
    let expected = Statement::Instruction("DRW".to_string(),
                                          vec![Operand::Register(Reg::V0),
                                               Operand::Register(Reg::Va),
                                               Operand::Literal(15)]);
    assert_eq!(result, Ok((expected, "")))
}

#[test]
fn test_operand_literal() {
    let result = parser(operand).parse("1312");
    let expected = Operand::Literal(1312);

    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_label_only_letters() {
    let result = parser(label).parse("hello: CLS");
    assert_eq!(result, Ok((Label("hello".to_string()), " CLS")));
}

#[test]
fn test_label_with_underscore() {
    let result = parser(label).parse("hello_world: CLS");
    assert_eq!(result, Ok((Label("hello_world".to_string()), " CLS")));
}

#[test]
fn test_label_with_uppercase() {
    let result = parser(label).parse("Label:");
    assert_eq!(result, Ok((Label("Label".to_string()), "")));
}

#[test]
fn compile_simple_instruction() {
    let compiled: Box<[u8]> = compile("CALL 228");
    let expected: Box<[u8]> = vec![0x22, 0x28].into_boxed_slice();

    assert_eq!(compiled, expected);
}
