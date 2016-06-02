use combine::{Parser, ParserExt, parser};
use combine::primitives::{State, Stream, ParseResult};

// TODO: Define own?
use vm::instruction::Reg;
use vm::instruction::Addr;

#[derive(Debug, PartialEq)]
pub enum Operand {
    Literal(u16),
    Label(String),
    Register(Reg),
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Label(String),
    Instruction(String, Vec<Operand>),
}

pub fn parse_source(source: &str) -> Vec<Statement> {
    // TODO: Error handling
    parser(stmts).parse(source).map(|x| x.0).unwrap()
}

fn stmts<I>(input: State<I>) -> ParseResult<Vec<Statement>, I>
    where I: Stream<Item = char>
{
    use combine::{many, try, newline};

    let stmt_parser = try(parser(stmt)).map::<_, Option<Statement>>(Some);
    let empty_line = try(newline::<I>()).map::<_, Option<Statement>>(|_| None);

    let stmt_or_new_line = stmt_parser.or(empty_line);
    let mut opt_stms = many(stmt_or_new_line).map::<_, Vec<Statement>>(|x| flatten_vec(x));

    opt_stms.parse_state(input)
}

fn flatten_vec<T>(v: Vec<Option<T>>) -> Vec<T> {
    v.into_iter().flat_map(|x| x.map(|y| vec![y]).unwrap_or_default()).collect()
}

fn stmt<I>(input: State<I>) -> ParseResult<Statement, I>
    where I: Stream<Item = char>
{
    use combine::{spaces, try};

    let label_parser = spaces().with(parser(label));
    let instruction_parser = spaces().with(parser(instruction));

    try(label_parser).or(instruction_parser).parse_state(input)
}

fn label<I>(input: State<I>) -> ParseResult<Statement, I>
    where I: Stream<Item = char>
{
    use combine::{many1, letter, token};

    let ident = many1(letter().or(token('_')));
    let label = ident.map(Statement::Label);

    label.skip(token(':')).parse_state(input)
}

fn instruction<I>(input: State<I>) -> ParseResult<Statement, I>
    where I: Stream<Item = char>
{
    use std::ascii::AsciiExt;
    use combine::{many1, letter, char, spaces, sep_by, parser, optional, between};

    let mnemonic =
        many1(letter()).map(|x: String| x.to_ascii_uppercase()).message("mnemonic expected");

    let lex_char = |c| char(c);
    let operands = sep_by(between(spaces(), spaces(), parser(operand::<I>)),
                          lex_char(','));

    let opt_operands = optional(between(spaces(), spaces(), operands));

    mnemonic.and(opt_operands)
        .map(|x| Statement::Instruction(x.0, x.1.unwrap_or_default()))
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
fn test_stmts_empty() {
    let result = parser(stmts).parse("\n\n");
    let expected = vec![];
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_stmts_with_nl() {
    let result = parser(stmts).parse("start:\nCLS\n");
    let expected = vec![Statement::Label("start".to_string()),
                        Statement::Instruction("CLS".to_string(), vec![])];
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_stmts_label_insn_nl() {
    let result = parser(stmts).parse("start: CLS\n");
    let expected = vec![Statement::Label("start".to_string()),
                        Statement::Instruction("CLS".to_string(), vec![])];
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_stmts_label_insn_operands_nl() {
    let result = parser(stmts).parse("start: CALL 520\n");
    let expected = vec![Statement::Label("start".to_string()),
                        Statement::Instruction("CALL".to_string(), vec![Operand::Literal(0x208)])];
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_stmts_with_new_lines() {
    let result = parser(stmts).parse("start:\nCLS\nhello: CALL 520");
    let expected = vec![Statement::Label("start".to_string()),
                        Statement::Instruction("CLS".to_string(), vec![]),
                        Statement::Label("hello".to_string()),
                        Statement::Instruction("CALL".to_string(), vec![Operand::Literal(0x208)])];
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_stmt_consume_label() {
    let result = parser(stmt).parse("hello: CLS");
    let expected = Statement::Label("hello".to_string());
    assert_eq!(result, Ok((expected, " CLS")));
}

#[test]
fn test_stmt_consume_label_lead_space() {
    let result = parser(stmt).parse(" hello: CLS");
    let expected = Statement::Label("hello".to_string());
    assert_eq!(result, Ok((expected, " CLS")));
}

#[test]
fn test_stmt_leading_spaces() {
    let result = parser(stmt).parse(" CLS");
    let expected = Statement::Instruction("CLS".to_string(), vec![]);
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_stmt_consume_newline() {
    let result = parser(stmt).parse(" CLS\n");
    let expected = Statement::Instruction("CLS".to_string(), vec![]);
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_stmt_consume_newline_with_spaces() {
    let result = parser(stmt).parse("CLS  \n");
    let expected = Statement::Instruction("CLS".to_string(), vec![]);
    assert_eq!(result, Ok((expected, "")));
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
    let expected = Statement::Instruction("SYS".to_string(), vec![Operand::Literal(228)]);
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_instruction_with_operand_tab_spaced() {
    let result = parser(instruction).parse("SYS\t228");
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
fn test_instruction_with_spaced_operands_after_comma() {
    let result = parser(instruction).parse("DRW     V0,\tVa,15");
    let expected = Statement::Instruction("DRW".to_string(),
                                          vec![Operand::Register(Reg::V0),
                                               Operand::Register(Reg::Va),
                                               Operand::Literal(15)]);
    assert_eq!(result, Ok((expected, "")))
}

#[test]
fn test_instruction_with_spaced_operands_before_comma() {
    let result = parser(instruction).parse("DRW     V0 ,Va\t,15");
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
    assert_eq!(result, Ok((Statement::Label("hello".to_string()), " CLS")));
}

#[test]
fn test_label_with_underscore() {
    let result = parser(label).parse("hello_world: CLS");
    assert_eq!(result,
               Ok((Statement::Label("hello_world".to_string()), " CLS")));
}

#[test]
fn test_label_with_uppercase() {
    let result = parser(label).parse("Label:");
    assert_eq!(result, Ok((Statement::Label("Label".to_string()), "")));
}
