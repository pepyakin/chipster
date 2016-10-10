use combine::{Parser, ParserExt, parser};
use combine::primitives::{State, Stream, ParseResult};

use parse::Statement;
use parse::Operand;
use parse::horizontal_spaces;

use parse::stmt::stmt;

pub fn program<'a, I: 'a>(input: State<I>) -> ParseResult<Vec<Statement>, I>
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

#[cfg(test)]
mod tests {
    use super::*;
    use parse::*;
    use combine::{Parser, ParserExt, parser};

    #[test]
    fn test_stmts_empty() {
        let result = parser(program).parse("\n\n");
        let expected = vec![];
        assert_eq!(result, Ok((expected, "")));
    }

    #[test]
    fn test_stmts_empty_with_spaces() {
        let result = parser(program).parse("\n \n");
        let expected = vec![];
        assert_eq!(result, Ok((expected, "")));
    }

    #[test]
    fn test_stmts_with_nl() {
        let result = parser(program).parse("start:\nCLS\n");
        let expected = vec![Statement::Label("start".to_string()),
                            Statement::Instruction("CLS".to_string(), vec![])];
        assert_eq!(result, Ok((expected, "")));
    }

    #[test]
    fn test_stmts_label_insn_nl() {
        let result = parser(program).parse("start: CLS\n");
        let expected = vec![Statement::Label("start".to_string()),
                            Statement::Instruction("CLS".to_string(), vec![])];
        assert_eq!(result, Ok((expected, "")));
    }

    #[test]
    fn test_stmts_label_insn_operands_nl() {
        let result = parser(program).parse("start: CALL 520\n");
        let expected = vec![Statement::Label("start".to_string()),
                            Statement::Instruction("CALL".to_string(),
                                                   vec![Operand::new_literal(0x208)])];
        assert_eq!(result, Ok((expected, "")));
    }

    #[test]
    fn test_stmts_with_new_lines() {
        let result = parser(program).parse("start:\nCLS\nhello: CALL 520");
        let expected = vec![Statement::Label("start".to_string()),
                            Statement::Instruction("CLS".to_string(), vec![]),
                            Statement::Label("hello".to_string()),
                            Statement::Instruction("CALL".to_string(),
                                                   vec![Operand::new_literal(0x208)])];
        assert_eq!(result, Ok((expected, "")));
    }
}
