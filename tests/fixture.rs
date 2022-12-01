use std::path::PathBuf;
use swc_core::ecma::transforms::testing::{test_fixture, FixtureTestConfig};
use swc_core::ecma::visit::as_folder;
use swc_ecma_parser::{EsConfig, Syntax};

use testing::fixture;

use next_superjson::{app::transform_app, page::transform_page, Config};

#[fixture("tests/fixture/page/**/code.js")]
fn fixture_page(input: PathBuf) {
    let output = input.with_file_name("output.js");

    test_fixture(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        &|_| {
            as_folder(transform_page(Config {
                excluded: vec!["smth".to_string()],
            }))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}

#[fixture("tests/fixture/app/**/code.js")]
fn fixture_app(input: PathBuf) {
    let output = input.with_file_name("output.js");

    test_fixture(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        &|_| {
            as_folder(transform_app(Config {
                excluded: vec!["smth".to_string()],
            }))
        },
        &input,
        &output,
        FixtureTestConfig {
            ..Default::default()
        },
    );
}
