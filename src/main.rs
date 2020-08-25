#![allow(non_snake_case)]

mod lib;
mod vdom;
#[cfg(test)]
mod tests;

/*
TODO - Dodać tranzakcyjną aktualizację
    self.deps.triggerChange([id1, id2, id3]);               //to powinno wystarczyc

TODO - Graph - usunac nieuzywane krawedzie (subskrybcje)

TODO - Graph - zamienić Clone na Copy

TODO - dodać jakieś makra które pozwolą na łatwe generowanie html-a (https://docs.rs/maplit/1.0.2/maplit/)
*/

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

fn main() {
    env_logger::init();

    println!("test");


    use crate::{
        lib::{
            Dependencies::Dependencies,
        },
        vdom::{
            models::{
                VDom::VDom,
            },
            startApp::startApp,
        }
    };


    struct AppState {}

    impl AppState {
        fn new(root: &Dependencies) -> AppState {
            AppState {
            }
        }
    }


    //po wystartowaniu subskrybcjaApp tą zmienną trzeba wpakować w zmienną globalną zeby nie stracić subskrybcji

    fn glownaFunkcjaRenderujaca(appState: &AppState) -> Vec<VDom> {

        use std::collections::HashMap;
        use crate::{
            vdom::{
                models::{
                    VDomNode::VDomNode,
                    VDomText::VDomText,
                },
            }
        };

        /*
            zaimplementować FromStr, albo coś takiego
            po to zeby dalo sie umiescic String lub str

            https://docs.rs/maplit/1.0.2/maplit/
        */

        vec!(
            VDom::Node {
                node: VDomNode {
                    name: "div".into(),
                    attr: map!{ "aaa".into() => "one".into(), "bbb".into() => "two".into() },
                    child: vec!(
                        VDom::Text {
                            node: VDomText{
                                value: "Abudabi".into(),
                            }
                        }
                    )
                }
            }
        )

        /*
        <div>
            "Abudabi"
        </div>
        */
    }



    let root: Dependencies = Dependencies::new();
    let appState = AppState::new(&root);

    let subskrybcjaApp = startApp(root, appState, glownaFunkcjaRenderujaca);

    println!("--- Wygaslo ---");

    subskrybcjaApp.off();
}
