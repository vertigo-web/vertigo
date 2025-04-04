use crate::{
    self as vertigo, component, css, dom, inspect::{log_start, DomDebugFragment}, AttrGroup,
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
        "<div alt='4' id='my_id'><span style='color: red'>Hello world</span></div>"
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

    assert_eq!(el_str, "<div><span>Hello world</span></div>");
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
        "<div style='color: red'><span style='color: green'>Hello world</span></div>"
    );
}

#[test]
fn test_on_events_grouping_and_spreading() {
    use crate::{self as vertigo, component, css, dom};

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
            inner:on_click={|| ()}
            inner:on_dropfile={|_| ()}
            inner:on_input={|_| ()}
            inner:on_key_down={|_| true}
            inner:on_load={|| ()}
            inner:on_mouse_down={|| true}
            inner:on_mouse_enter={|| ()}
            inner:on_mouse_leave={|| ()}
            inner:on_mouse_up={|| true}
            inner:on_submit={|| ()}
            inner:vertigo-suspense={|_| css!{"color: red;"}}
        />
    };

    let el_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(el_str, "<input name='world' style='color: red' blur=2 change=3 click=4 drop=5 hook_keydown=1 input=6 keydown=7 load=8 mousedown=9 mouseenter=10 mouseleave=11 mouseup=12 submit=13 />");
}
