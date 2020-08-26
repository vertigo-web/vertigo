#![allow(non_snake_case)]

/*
TODO - Dodać tranzakcyjną aktualizację
    self.deps.triggerChange([id1, id2, id3]);               //to powinno wystarczyc

TODO - Graph - usunac nieuzywane krawedzie (subskrybcje)

TODO - Graph - zamienić Clone na Copy

TODO - dodać jakieś makra które pozwolą na łatwe generowanie html-a (https://docs.rs/maplit/1.0.2/maplit/)


TODO - Będąc w bloku computed, albo subskrybcji, całkowicie ignorować wszelkie akcje które będą chciały zmienić wartość
       rzucać standardowy strumień błędów informację o incydencie. Dzięki temu nowa wadliwa funkcjonalność nie zepsuje tej juz dobrze ulezanej funkcjonalności
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

    use std::rc::Rc;
    use virtualdom::computed::{
        Dependencies::Dependencies,
        Value::Value,
    };

    use virtualdom::{
        vdom::{
            models::{
                VDom::VDom,
            },
            startApp::startApp,
        }
    };


    struct AppState {
        value: Value<u32>,
        at: Value<u32>
    }

    impl AppState {
        fn new(root: &Dependencies) -> Rc<AppState> {
            Rc::new(AppState {
                value: root.newValue(33),
                at: root.newValue(999),
            })
        }
    }


    //po wystartowaniu subskrybcjaApp tą zmienną trzeba wpakować w zmienną globalną zeby nie stracić subskrybcji

    fn glownaFunkcjaRenderujaca(appState: &Rc<AppState>) -> Vec<VDom> {

        let appState = appState.clone();
        use virtualdom::vdom::{
            models::{
                VDomNode::VDomNode,
                VDomText::VDomText,
            },
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
                    attr: map!{ "aaa".into() => "one".into(), "bbb".into() => format!("wallll {}", appState.at.getValue()) },
                    child: vec!(
                        VDom::Text {
                            node: VDomText{
                                value: format!("aktualna wartosc = {}", appState.value.getValue()),
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



    let root: Dependencies = Dependencies::new();
    let appState = AppState::new(&root);

    let subskrybcjaApp = startApp(root, appState.clone(), glownaFunkcjaRenderujaca);

    appState.value.setValue(55);
    println!("Przestawiam atrybut");
    appState.at.setValue(1000);

    println!("--- Wygaslo ---");

    subskrybcjaApp.off();
}
