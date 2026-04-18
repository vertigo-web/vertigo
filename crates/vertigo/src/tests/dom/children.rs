use crate::dev::inspect::{DomDebugFragment, log_start};

#[test]
fn test_children() {
    use crate::{self as vertigo, DomNode, component, dom};

    #[component]
    fn Hello<'a>(name: &'a str) {
        dom! {
            <span>"Hello " {name}</span>
        }
    }

    #[component]
    fn Sup<'a>(name: &'a str) {
        dom! {
            <span>"Sup " {name} "?"</span>
        }
    }

    #[component]
    fn Wrapper<'a>(name: &'a str, children: Vec<DomNode>) {
        dom! {
            <div>
                "Wrapper for " {name}
                {..children}
            </div>
        }
    }

    log_start();

    let _ret = dom! {
        <Wrapper name={"world"}>
            <Hello name={"world"} />
            <Sup name={"world"} />
        </Wrapper>
    };

    let el_str = DomDebugFragment::from_log().to_pseudo_html();

    assert_eq!(
        el_str,
        "<div v-component='Wrapper'>Wrapper for world<span v-component='Hello'>Hello world</span><span v-component='Sup'>Sup world?</span></div>"
    );
}
