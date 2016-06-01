extern crate combine;

use combine::{between, char, letter, spaces, many1, parser, sep_by, token, Parser, ParserExt};
use combine::primitives::{State, Stream, ParseResult};

#[derive(Debug, PartialEq)]
struct Label(String);

enum Operand {
    Literal(u16),
    Label(String),
    Register,
}

struct Statement {
    label: Label,
    mnemonic: String,
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
    let ident = many1(letter().or(token('_')));
    let label = ident.map(Label);

    label.skip(token(':')).parse_state(input)
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
    let compiled: Box<[u8]> = compile("CALL #228");
    let expected: Box<[u8]> = vec![0x22, 0x28].into_boxed_slice();

    assert_eq!(compiled, expected);
}
