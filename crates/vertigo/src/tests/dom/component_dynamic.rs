use crate::{
    self as vertigo, component, css,
    dev::inspect::{log_start, DomDebugFragment},
    dom, AttrGroup, Computed, EmbedDom,
};

#[test]
fn test_attr_values_grouping_and_spreading() {
    #[component]
    fn Hello<'a>(name: &'a str, div: AttrGroup, span: AttrGroup) {
        dom! {
            <div {..div}>
                <span {..span}>
                    "Hello " {name}
                </span>
            </div>
        }
    }

    log_start();

    let _el = dom! {
        <Hello
            name="world"
            div:id="my_id"
            div:alt={2+2}
            span:style={format!("color: {}", "red")}
        />
    };

    let el_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(
        el_str,
        "<div alt='4' id='my_id' v-component='Hello'><span style='color: red'>Hello world</span></div>"
    );
}

#[test]
fn test_empty_attrs_grouping() {
    #[component]
    fn Hello<'a>(name: &'a str, div: AttrGroup) {
        dom! {
            <div {..div}>
                <span>
                    "Hello " {name}
                </span>
            </div>
        }
    }

    log_start();

    let _el = dom! {
        <Hello
            name="world"
        />
    };

    let el_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(
        el_str,
        "<div v-component='Hello'><span>Hello world</span></div>"
    );
}

#[test]
fn test_css_attrs_grouping_and_spreading() {
    #[component]
    fn Hello<'a>(name: &'a str, div: AttrGroup, span: AttrGroup) {
        dom! {
            <div {..div}>
                <span {..span}>
                    "Hello " {name}
                </span>
            </div>
        }
    }

    let red_css = css!("color: red;");
    let green_css = css!("color: green;");

    log_start();

    let _el = dom! {
        <Hello
            name="world"
            div:css={red_css}
            span:css={green_css}
        />
    };

    let el_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(
        el_str,
        "<div style='color: red' v-component='Hello' v-css='red_css'><span style='color: green' v-css='green_css'>Hello world</span></div>"
    );
}

#[test]
fn test_css_extending() {
    use vertigo::{component, css, dom, AttrGroup, Value};

    #[component]
    fn MyInput(value: Value<String>, inner: AttrGroup) {
        let css = css!("color: red; background-color: green;");
        dom! { <input value={value} {css} {..inner} /> }
    }

    let my_value = Value::new("Test".to_string());
    let my_css = css! {"color: yellow; font-size: 15px;"};

    log_start();

    let _el = dom! {
        <MyInput value={my_value} inner:id="test-id" inner:css={my_css} />
    };

    let el_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(
        el_str,
        "<input id='test-id' style='color: red; background-color: green; color: yellow; font-size: 15px' v-component='MyInput' v-css='my_css' value='Test' />"
    );
}

#[test]
fn test_on_events_grouping_and_spreading() {
    use crate::{self as vertigo, component, dom};

    #[component]
    fn Everything<'a>(name: &'a str, inner: AttrGroup) {
        dom! {
            <input {name} {..inner} />
        }
    }

    log_start();

    let _el = dom! {
        <Everything
            name="world"
            inner:hook_key_down={|_| true}
            inner:on_blur={|| ()}
            inner:on_change={|_| ()}
            inner:on_click={|_| ()}
            inner:on_dropfile={|_| ()}
            inner:on_input={|_| ()}
            inner:on_key_down={|_| true}
            inner:on_load={|| ()}
            inner:on_mouse_down={|| true}
            inner:on_mouse_enter={|| ()}
            inner:on_mouse_leave={|| ()}
            inner:on_mouse_up={|| true}
            inner:on_submit={|| ()}
        />
    };

    let el_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(el_str, "<input name='world' v-component='Everything' blur=2 change=3 click=4 drop=5 hook_keydown=1 input=6 keydown=7 load=8 mousedown=9 mouseenter=10 mouseleave=11 mouseup=12 submit=13 />");
}

#[test]
fn test_stringifyable_group_attrs() {
    #[component]
    fn Hello<'a>(name: &'a str, opt: AttrGroup) {
        let label = opt
            .get("label")
            .map(|l| l.to_string_or_empty())
            .unwrap_or_else(|| Computed::from(|_| "Hello".to_string()));

        dom! {
            <div>
                <span>
                    {label} " " {name}
                </span>
            </div>
        }
    }

    log_start();

    let _el1 = dom! {
        <Hello
            name="world"
        />
    };

    let el1_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(
        el1_str,
        "<div v-component='Hello'><span>Hello<!-- v --> world</span></div>"
    );

    log_start();

    let _el2 = dom! {
        <Hello
            name="world"
            opt:label={"Good bye"}
        />
    };

    let el2_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(
        el2_str,
        "<div v-component='Hello'><span>Good bye<!-- v --> world</span></div>"
    );
}

#[test]
fn test_embeddable_group_attrs() {
    #[component]
    fn Hello<'a>(name: &'a str, mut opt: AttrGroup) {
        let label = opt
            .get("label")
            .map(EmbedDom::embed)
            .unwrap_or("Hello".embed());

        let id = opt.remove_entry("id");

        dom! {
            <div {..id}>
                <span>
                    {label} " " {name}
                </span>
            </div>
        }
    }

    log_start();

    let _el1 = dom! {
        <Hello
            name="world"
        />
    };

    let el1_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(
        el1_str,
        "<div v-component='Hello'><span>Hello world</span></div>"
    );

    log_start();

    let _el2 = dom! {
        <Hello
            name="world"
            opt:id="my_div"
            opt:label="Good bye"
        />
    };

    let el2_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(
        el2_str,
        "<div id='my_div' v-component='Hello'><span>Good bye<!-- v --> world</span></div>"
    );
}

#[test]
fn test_embeddable_group_attrs_cloned() {
    #[component]
    fn Hello(opt: AttrGroup) {
        let id = opt.get_key_value("id");

        dom! {
            <div {..id}>
                <span />
            </div>
        }
    }

    log_start();

    let _el2 = dom! {
        <Hello
            opt:id="my_div"
        />
    };

    let el2_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(
        el2_str,
        "<div id='my_div' v-component='Hello'><span /></div>"
    );
}
