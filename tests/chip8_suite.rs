extern crate vm;
extern crate test;

use std::{fs, io, env};
use std::path::Path;
use std::collections::{HashSet, HashMap};

use test::{TestDesc, TestDescAndFn, DynTestName, TestFn};

fn collect_tests() { }

fn main() {
    let args: Vec<_> = env::args().collect();
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    println!("{:?}", src_dir);

    test::test_main(&args, vec![TestDescAndFn {
        desc: TestDesc {
            name: DynTestName("foo".to_string()),
            ignore: false,
            should_panic: test::ShouldPanic::No,
        },
        testfn: TestFn::dyn_test_fn(move || {
            panic!("test panic!");
        }),
    }]);
}
