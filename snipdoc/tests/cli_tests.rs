#[test]
fn cli_tests() {
    let t = trycmd::TestCases::new();
    // t.case("tests/cmd/*.trycmd");
    t.case("tests/cmd/check-valid.trycmd");
}

#[cfg(feature = "exec")]
#[test]
fn cli_tests_with_exec_feature() {
    let t = trycmd::TestCases::new();
    t.case("tests/exec-feature/*.trycmd");
}
