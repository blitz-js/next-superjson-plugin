use std::path::PathBuf;
use swc_core::ecma::transforms::testing::test_fixture;
use swc_ecma_parser::{EsConfig, Syntax};

use testing::fixture;

use next_superjson;
use next_superjson::Config;

//use std::env;

#[fixture("tests/fixture/**/code.js")]
fn fixture(input: PathBuf) {
    let output = input.with_file_name("output.js");

    //env::set_var("UPDATE", "1");

    test_fixture(
        Syntax::Es(EsConfig {
            //jsx: input.to_string_lossy().ends_with(".jsx"),
            jsx: true,
            ..Default::default()
        }),
        &|_| {
            next_superjson::plugin(Config {
                excluded: vec!["smth".to_string()],
            })
        },
        &input,
        &output,
    );
}
