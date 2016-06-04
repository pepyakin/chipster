#![feature(slice_patterns)]
#![feature(concat_idents)]

extern crate combine;
extern crate vm;
extern crate byteorder;

mod parse;
mod translate;

pub fn compile(source: &str) -> Box<[u8]> {
    let statements = parse::parse_source(source);
    translate::translate(statements)
}

macro_rules! check_instruction {
    (
        name: $test_name:ident,
        source: $source:expr,
        expected: $expected:expr
    ) => {
        #[test]
        fn $test_name() {
            use byteorder::{ByteOrder, BigEndian};
            use std::io::Read;
            
            let mut buf = [0u8; 2];
            BigEndian::write_u16(&mut buf, $expected);
            
            let compiled: Vec<u8> = compile($source).into_vec();
            let expected: Vec<u8> = buf.to_vec();
            assert_eq!(compiled, expected);
        }
    };
}

check_instruction! {
    name: cls,
    source: "CLS",
    expected: 0x00E0
}

check_instruction! {
    name: ret,
    source: "RET",
    expected: 0x00EE
}

check_instruction! {
    name: jump,
    source: "JP 32",
    expected: 0x1020
}

check_instruction! {
    name: call,
    source: "CALL 32",
    expected: 0x2020
}

check_instruction! {
    name: se_reg_imm,
    source: "SE V1, 9",
    expected: 0x3109
}

check_instruction! {
    name: sne_reg_imm,
    source: "SNE V3, 32",
    expected: 0x4320
}

check_instruction! {
    name: se_reg_reg,
    source: "SE V4, V5",
    expected: 0x5450
}

check_instruction! {
    name: put_imm,
    source: "LD V4, 16",
    expected: 0x6410
}

check_instruction! {
    name: add_imm,
    source: "ADD V0, 40",
    expected: 0x7028
}

check_instruction! {
    name: apply_id,
    source: "LD V2, V7",
    expected: 0x8270
}

check_instruction! {
    name: apply_or,
    source: "OR V2, V7",
    expected: 0x8271
}

check_instruction! {
    name: apply_and,
    source: "AND V2, V7",
    expected: 0x8272
}

check_instruction! {
    name: apply_xor,
    source: "XOR V2, V7",
    expected: 0x8273
}

check_instruction! {
    name: apply_add,
    source: "ADD V2, V7",
    expected: 0x8274
}

check_instruction! {
    name: apply_sub,
    source: "SUB V2, V7",
    expected: 0x8275
}

check_instruction! {
    name: apply_shr,
    source: "SHR V6, V6",
    expected: 0x8666
}

check_instruction! {
    name: apply_subn,
    source: "SUBN V2, V7",
    expected: 0x8277
}

check_instruction! {
    name: apply_shl,
    source: "SHL V2, V7",
    expected: 0x827E
}

check_instruction! {
    name: sne_reg_reg,
    source: "SNE V2, V7",
    expected: 0x9270
}

check_instruction! {
    name: ld_i_addr,
    source: "LD I, 20",
    expected: 0xA014
}

check_instruction! {
    name: jump_plus_v0,
    source: "JP V0, 20",
    expected: 0xB014
}

check_instruction! {
    name: randomize,
    source: "RND V2, #74",
    expected: 0xC274
}

check_instruction! {
    name: draw,
    source: "DRW V2, V7, 4",
    expected: 0xD274
}

check_instruction! {
    name: skp,
    source: "SKP V2",
    expected: 0xE29E
}

check_instruction! {
    name: sknp,
    source: "SKNP V2",
    expected: 0xE2A1
}

check_instruction! {
    name: ld_reg_dt,
    source: "LD V2, DT",
    expected: 0xF207
}

check_instruction! {
    name: ld_reg_k,
    source: "LD V2, K",
    expected: 0xF20A
}

check_instruction! {
    name: ld_dt_reg,
    source: "LD DT, V2",
    expected: 0xF215
}

check_instruction! {
    name: ld_st_reg,
    source: "LD ST, V2",
    expected: 0xF218
}

check_instruction! {
    name: ld_add_i,
    source: "ADD I, V2",
    expected: 0xF21E
}

check_instruction! {
    name: ld_load_glpyh,
    source: "LD F, V2",
    expected: 0xF229
}

check_instruction! {
    name: ld_b_reg,
    source: "LD B, V2",
    expected: 0xF233
}

check_instruction! {
    name: ld_deref_i_reg,
    source: "LD [I], V2",
    expected: 0xF255
}

check_instruction! {
    name: ld_reg_deref_i,
    source: "LD V2, [I]",
    expected: 0xF265
}
