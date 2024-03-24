#[test]
fn test_window() {
    use crate as vertigo;
    use crate::window;

    let x = 5;
    let foo: &str = "foo";
    let bar = "bar".to_string();

    window!(
        "aFunctionRichInArguments()",
        3,
        -34,
        "blablabla",
        true,
        false,
        34.56,
        x,
        -x,
        foo,
        bar,
    );
}
