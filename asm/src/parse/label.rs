use combine::{Parser, ParserExt, parser};
use combine::primitives::{State, Stream, ParseResult};

use parse::Statement;
use parse::Operand;

use parse::ident;

pub fn label<I>(input: State<I>) -> ParseResult<Statement, I>
    where I: Stream<Item = char>
{
    use combine::{token, try};

    let label = ident().map(Statement::Label);

    try(label).skip(token(':')).parse_state(input)
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