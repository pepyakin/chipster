use combine::{Parser, ParserExt, parser};
use combine::primitives::{State, Stream, ParseResult};

// TODO: Define own?
use vm::instruction::Reg;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LiteralValue {
    raw: u16,
}

impl LiteralValue {
    pub fn new(raw: u16) -> LiteralValue {
        LiteralValue { raw: raw }
    }

    pub fn as_u8(&self) -> u8 {
        assert!((self.raw & 0xFF) == self.raw);
        self.raw as u8
    }

    pub fn as_u16(&self) -> u16 {
        assert!((self.raw & 0xFFFF) == self.raw);
        self.raw as u16
    }
}

#[derive(Debug, PartialEq)]
pub enum Operand {
    Literal(LiteralValue),
    Label(String),
    Register(Reg),

    /// I
    IndexReg,

    /// [I]
    DerefIndexReg,

    /// F designator
    F,

    /// BCD designator
    B,

    /// Keyboard designator
    K,

    /// DT
    DT,

    // ST
    ST,
}

impl Operand {
    pub fn new_literal(value: u16) -> Operand {
        Operand::Literal(LiteralValue::new(value))
    }
    
    pub fn new_label(name: String) -> Operand {
        Operand::Label(name)
    }
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
    let mut opt_stms = many(stmt_or_new_line).map::<_, Vec<Statement>>(flatten_vec);

    opt_stms.parse_state(input)
}

fn flatten_vec<T>(v: Vec<Option<T>>) -> Vec<T> {
    v.into_iter().flat_map(|x| x.map(|y| vec![y]).unwrap_or_default()).collect()
}

fn stmt<I>(input: State<I>) -> ParseResult<Statement, I>
    where I: Stream<Item = char>
{
    use combine::try;

    let label_parser = parser(horizontal_spaces).with(parser(label));
    let instruction_parser = parser(horizontal_spaces).with(parser(instruction));

    try(label_parser).or(instruction_parser).parse_state(input)
}

fn label<I>(input: State<I>) -> ParseResult<Statement, I>
    where I: Stream<Item = char>
{
    use combine::{token, try};

    let label = parser(ident).map(Statement::Label);

    try(label).skip(token(':')).parse_state(input)
}

fn instruction<I>(input: State<I>) -> ParseResult<Statement, I>
    where I: Stream<Item = char>
{
    use std::ascii::AsciiExt;
    use combine::{many1, letter, char, sep_by, parser, optional, between};

    let mnemonic =
        many1(letter()).map(|x: String| x.to_ascii_uppercase()).message("mnemonic expected");

    let operands = sep_by(between(parser(horizontal_spaces),
                                  parser(horizontal_spaces),
                                  parser(operand::<I>)),
                          char(','));

    let opt_operands = optional(between(parser(horizontal_spaces),
                                        parser(horizontal_spaces),
                                        operands));

    mnemonic.and(opt_operands)
        .map(|x| Statement::Instruction(x.0, x.1.unwrap_or_default()))
        .parse_state(input)
}

fn horizontal_spaces<I>(input: State<I>) -> ParseResult<(), I>
    where I: Stream<Item = char>
{
    // TODO: Define custom parser
    use combine::{tab, char, skip_many};

    skip_many(tab().or(char(' '))).map(|_| ()).parse_state(input)
}

fn operand<I>(input: State<I>) -> ParseResult<Operand, I>
    where I: Stream<Item = char>
{
    use combine::{many1, digit, token, hex_digit, string};

    let literal = parser(|input| {
        many1(digit())
            .and_then(|s: String| s.parse::<u16>())
            .map(Operand::new_literal)
            .parse_state(input)
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
    let label = parser(ident).map(Operand::new_label);

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

fn ident<I>(input: State<I>) -> ParseResult<String, I>
    where I: Stream<Item = char>
{
    use combine::{many1, letter, token};
    many1(letter().or(token('_'))).parse_state(input)
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
                        Statement::Instruction("CALL".to_string(),
                                               vec![Operand::new_literal(0x208)])];
    assert_eq!(result, Ok((expected, "")));
}

#[test]
fn test_stmts_with_new_lines() {
    let result = parser(stmts).parse("start:\nCLS\nhello: CALL 520");
    let expected = vec![Statement::Label("start".to_string()),
                        Statement::Instruction("CLS".to_string(), vec![]),
                        Statement::Label("hello".to_string()),
                        Statement::Instruction("CALL".to_string(),
                                               vec![Operand::new_literal(0x208)])];
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
fn test_stmt_dont_consume_newline() {
    let result = parser(stmt).parse(" CLS\n");
    let expected = Statement::Instruction("CLS".to_string(), vec![]);
    assert_eq!(result, Ok((expected, "\n")));
}

#[test]
fn test_stmt_dont_consume_newline_with_spaces() {
    let result = parser(stmt).parse("CLS  \n");
    let expected = Statement::Instruction("CLS".to_string(), vec![]);
    assert_eq!(result, Ok((expected, "\n")));
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
fn test_operand_literal() {
    let result = parser(operand).parse("1312");
    let expected = Operand::new_literal(1312);

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
