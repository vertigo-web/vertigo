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
*/

fn main() {
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
        todo!();
    }



    let root: Dependencies = Dependencies::new();
    let appState = AppState::new(&root);

    let subskrybcjaApp = startApp(root, appState, glownaFunkcjaRenderujaca);

    println!("Wygaslo");

    subskrybcjaApp.off();
}
