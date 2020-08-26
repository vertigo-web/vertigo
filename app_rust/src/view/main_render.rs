
use std::rc::Rc;

use virtualdom::{
    vdom::{
        models::{
            VDom::VDom,
            VDomNode::VDomNode,
            VDomText::VDomText,
        },
    },
};

use crate::app_state::AppState;

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);


pub fn main_render(app_state: &Rc<AppState>) -> Vec<VDom> {

    let app_state = app_state.clone();

    /*
        zaimplementować FromStr, albo coś takiego
        po to zeby dalo sie umiescic String lub str

        https://docs.rs/maplit/1.0.2/maplit/
    */

    vec!(
        VDom::Node {
            node: VDomNode {
                name: "div".into(),
                attr: map!{ "aaa".into() => "one".into(), "bbb".into() => format!("wallll {}", app_state.at.getValue()) },
                child: vec!(
                    VDom::Text {
                        node: VDomText {
                            value: format!("aktualna wartosc = {}", app_state.value.getValue()),
                        }
                    }
                )
            }
        }
    )

    /*
    <div aaa="one" bbb="two">
        "Abudabi"
    </div>
    */
}
