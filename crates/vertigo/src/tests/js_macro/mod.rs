use crate::{tests::js_macro::api_mock::ApiMock};

mod api_mock;

#[test]
fn test_method_call() {
    use api_mock as vertigo;
    use crate::js;

    let pos: i32 = 100;

    let result = js! {
        window.scrollTo(pos)
    };

    result.expect(
        r#"
            vertigo::get_driver()
                .api_access()
                .root("window")
                .call("scrollTo", [I32(100)])
                .fetch()
        "#
    );
}

#[test]
fn test_property() {
    use api_mock as vertigo;
    use crate::js;

    let result = js! {
        window.referrer
    };

    result.expect(
        r#"
            vertigo::get_driver()
                .api_access()
                .root("window")
                .get("referrer")
                .fetch()
        "#
    );
}

#[test]
fn test_complex_receiver() {
    use api_mock as vertigo;
    use crate::js;

    let node = "node_mock";

    let result = js! {
        document.getElementById("foo").firstChild.appendChild(node)
    };

    result.expect(
        r#"
            vertigo::get_driver()
                .api_access()
                .root("document")
                .call("getElementById", [String("foo")])
                .get("firstChild")
                .call("appendChild", [String("node_mock")])
                .fetch()
        "#
    );
}

#[test]
fn test_ref() {
    use crate::js;

    let node_ref = ApiMock::new_ref(5);

    let result = js! {
        #node_ref.firstChild.focus()
    };

    result.expect(
        r#"
            node_ref(5)
                .api_access()
                .get("firstChild")
                .call("focus", [])
                .fetch()
        "#
    );
}

#[test]
fn test_many_arguments() {
    use api_mock as vertigo;
    use crate::js;

    let x = 5;
    let foo: &str = "foo";
    let bar = "bar".to_string();

    let result = js! {
        window.aFunctionRichInArguments(3, -34, "blablabla", true, false, 34.56, x, -x, foo, bar)
    };

    result.expect(
        r#"
            vertigo::get_driver()
                .api_access()
                .root("window")
                .call("aFunctionRichInArguments", [I32(3), I32(-34), String("blablabla"), True, False, F64(34.56), I32(5), I32(-5), String("foo"), String("bar")])
                .fetch()
        "#
    );
}
