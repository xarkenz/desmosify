mod common;

simple_test!(test_strings, "strings", "desmos-graphing");

#[test]
fn test_docs_example() {
    common::run_test(
        [
            "docs_example/fibonacci.desmos",
        ],
        "docs_example/out/docs_example.json",
        "desmos-graphing",
    )
}
