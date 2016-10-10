use combine::{Parser, ParserExt, parser};
use combine::primitives::{State, Stream, ParseResult};

use parse::Statement;
use parse::Operand;
use parse::horizontal_spaces;
use parse::instruction::instruction;
use parse::label::label;

use vm::instruction::Reg;

pub fn stmt<'a, I: 'a>(input: State<I>) -> ParseResult<Statement, I>
    where I: Stream<Item = char>
{
    use combine::try;

    let label_parser = horizontal_spaces().with(parser(label));
    let instruction_parser = horizontal_spaces().with(parser(instruction));

    try(label_parser).or(instruction_parser).parse_state(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::{Parser, ParserExt, parser};
    use combine::primitives::{State, Stream, ParseResult};
    use parse::Statement;
    use parse::Operand;
    use parse::horizontal_spaces;
    use parse::instruction::instruction;

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
}
