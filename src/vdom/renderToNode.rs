
use std::rc::Rc;

use crate::{
    lib::{
        Client::Client,
        Computed::Computed,
        Dependencies::Dependencies,
    },
    vdom::{
        models::{
            Component::{
                Component,
            },
            Handler::{Handler, HandlerTarget},
            VDom::{VDom, VDomNode},
            RealDomNode::RealDomNode,
            RealDom::RealDom,
        }
    }
};

/*
    Główny root aplikacji, powinien być niezmiennym i niemodyfikowalnym węzłem
    Od niego zaczynamy zawsze (numer 1)
*/

fn applyNewViewChild(anchor: HandlerTarget, a: &mut Vec<RealDom>, b: Rc<Vec<VDom>>) -> Vec<RealDom> {

    /*
        teraz kwestia jak zsynchronizować te dzieci

        Component-y reuzywamy

        najpierw porządkujemy koleność
            przenoszenie
            tworzenie nowych
            kasowanie nieaktualnych


        synchronizujac kompoment, trzeba na nowo nadac mu jego
        handler: RenderedHandler

        To będzie potrzebne dla tego komponentu gdy będzie musiał się przerenderować
    */
    todo!();
}


fn applyNewViewNode(om_a: &RealDomNode, dom_b: &VDomNode) {
    /*
        zeby przystąpić do synchronizaczji dwóch elementów, typ węzła musi się zgadzać
            RealDom::name musi mieć takie samo jak VDom:name
        
        synchronizujemy atrybuty

        potem trzeba będzie zsynchronizować eventy podpięte pod ten węzeł

        potem przechodzimy do synchronizowania dzieci
    */
}



/*
AppDataState --- stan dotyczący danhych

AppViewState (wstrzyknięcie AppDataState) - stan dotyczący widoku

AppState {
    data: AppDataState,
    view: AppViewState,
}
*/



pub fn renderToNode(target: Handler, computed: Computed<Rc<Vec<VDom>>>) -> Client { 
    let subscription: Client = computed.subscribe(move |appVDom| {
        let anchor = target.targetToRender.get(|state| {
            state.clone()
        });

        target.child.change(
            (anchor, appVDom),
            |currentAppDom, (anchor, appVDom)| {
                applyNewViewChild(
                    anchor,
                    currentAppDom,
                    appVDom.clone()
                );
            }
        );
    });

    subscription
}



//Statyczna zmienna, która będzie miała wartość null lub ta zmienna

//Funkcja wyeksportowana, która wywołana ustai tą zmienną globalną
//Funkcja wyeksportowana, która wyłączy tą zmienną. Sprawdzić czy się destruktor poprawnie wywoła


//Trzeba jakoś zapisać referencję do tej subskrybcji





/*
    renderDom => {
        <div>....</div>
        { memo(renderStalyElement) }
        <div>....</div>
    }

    renderStalyElement = () : Vec<VDom> {
        <div>
            jaksis staly naglowek, który nie będzie ulegał przegenerowaniu
        </div>
    }
*/

/*
    wyrenderowanie głównego komponentu

    fn newComponent<T: Debug>(
        root: Dependencies,
        params: Computed<T>,
        render: fn(T) -> Vec<VDom>
    ) -> Component

    Trzeba będzie go teraz jakoś zaaplikować do węzłą o numerze 1.

    Trzeba stworzyc reprezentację węzła 1.

    applyNewViewNode
        pasuje zeby ta funkcja modyfikowala tylko RealDome
*/
