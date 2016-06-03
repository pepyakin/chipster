#![feature(slice_patterns)]

extern crate combine;
extern crate vm;

mod parse;
mod translate;

pub fn compile(source: &str) -> Box<[u8]> {
    let statements = parse::parse_source(source);
    translate::translate(statements)
}

#[test]
fn compile_unary_instruction() {
    let compiled: Box<[u8]> = compile("CALL 32");
    let expected: Box<[u8]> = vec![0x20, 0x20].into_boxed_slice();

    assert_eq!(compiled, expected);
}

#[test]
fn compile_binary_instruction() {
    let compiled: Box<[u8]> = compile("SE V0, Va");
    let expected: Box<[u8]> = vec![0x50, 0xA0].into_boxed_slice();

    assert_eq!(compiled, expected);
}

#[test]
fn compile_ld_i() {
    let compiled: Box<[u8]> = compile("LD I, 512");
    let expected: Box<[u8]> = vec![0xA2, 0x00].into_boxed_slice();

    assert_eq!(compiled, expected);
}
