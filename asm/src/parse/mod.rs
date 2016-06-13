mod program;
mod stmt;
mod instruction;
mod label;

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

    pub fn as_u4(&self) -> u8 {
        assert!((self.raw & 0x0F) == self.raw);
        self.raw as u8
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Label(String),
    Instruction(String, Vec<Operand>),
}

pub fn parse_source(source: &str) -> Vec<Statement> {
    // TODO: Error handling
    use self::program::program;

    parser(program).parse(source).map(|x| x.0).unwrap()
}

fn comment<'a, I: 'a>(input: State<I>) -> ParseResult<(), I>
    where I: Stream<Item = char>
{
    use combine::{token, skip_many, satisfy};
    (token(';'), skip_many(satisfy(|c| c != '\n'))).map(|_| ()).parse_state(input)
}

fn horizontal_spaces<'a, I>() -> Box<Parser<Input = I, Output = ()> + 'a>
    where I: Stream<Item = char> + 'a
{
    use combine::{tab, char, skip_many};
    Box::new(skip_many(tab().or(char(' '))).map(|_| ()))
}

fn ident<'a, I>() -> Box<Parser<Input = I, Output = String> + 'a>
    where I: Stream<Item = char> + 'a
{
    use combine::{many1, letter, token};
    Box::new(many1(letter().or(token('_'))))
}
