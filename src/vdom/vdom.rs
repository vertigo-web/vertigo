
use std::collections::HashMap;
use std::rc::Rc;

use crate::lib::{
    Client::Client,
    GraphId::GraphId,
    Computed::Computed,
    Dependencies::Dependencies,
    BoxRefCell::BoxRefCell,
};

#[derive(Clone)]
struct RealNodeId {
    id: u64,
}

impl RealNodeId {
    fn root() -> RealNodeId {
        RealNodeId {
            id: 1
        }
    }
}

struct ComponentId {
    idComputed: GraphId,        //id tego computed
    idFunction: u64,            //id tej konkretnej funkcji statycznej (renderującej komponent)
}

struct Component {
    id: ComponentId,
    render: Box<dyn Fn() -> Rc<Vec<VDom>>>
}

struct VDomNode {
    name: String,
    attr: HashMap<String, String>,
    child: Vec<VDom>,
}


enum VDom {
    Node {
        node: VDomNode,
    },
    Text {
        value: String,
    },
    Component {
        node: Component,
    },
}

struct RealDomNode {
    name: String,
    attr: HashMap<String, String>,
    child: Vec<RealDom>,
    idDom: u64,                             //id realnego doma
}

enum RealDom {
    Node {
        node: RealDomNode,
    },
    Text {
        value: String,
        idDom: u64,                             //id realnego doma
    },
    Component {
        id: ComponentId,
        subscription: Client,                   //Subskrybcją, kryje się funkcja, która odpalana (na zmianę), wstawia coś do pojemnika child
                                                //idParenta mozemy przekazywac w momencie gdy będziemy tworzyć Client-a

        child: BoxRefCell<Vec<RealDom>>,


        // child: Rc<{                             //Ten element będzie przekazany do funkcji renderującej ---> a potem subskrybcja będzie zapisana do zmiennej subscription
        //     child: BoxRefCell<Rc<Vec<RealDom>>>,
        //     idParent: RealNodeId,               //prawdopodobnie będzie konieczne. Ale ten id moze byc utworzony przy stworzeniu noda. Nie będzie zmieniany.
        // }>
    }
}

#[derive(Clone)]
enum DomAnchor {
    Parent(RealNodeId),             //oznacza ze zaczynamy wstawiac elementy jako pierwsze dziecko
    RefPrev(RealNodeId),            //pokazuje poprzedni element przed zakresem
}

impl DomAnchor {
    fn root() -> DomAnchor {
        DomAnchor::Parent(RealNodeId::root())
    }
}

/*
RealDom::Node - DomAnchor::Parent(), będzie odnosnikiem
RealDom::Component - DomAnchor::RefPrev()
*/


fn newComponent<T: 'static>(root: Dependencies, params: Computed<T>, render: fn(T) -> Vec<VDom>) -> Component {
    let clientId = 4;   //TODO
    //let getValue = root.wrapGetValue(render, clientId);
    // to trzeba zamienic na subksrybcje, trzeba wystawic jakas funkcje subskryubujaca na funkcje (autorun)
    todo!();
}

/*
    Główny root aplikacji, powinien być niezmiennym i niemodyfikowalnym węzłem
    Od niego zaczynamy zawsze (numer 1)
*/

fn applyNewViewChild(anchor: DomAnchor, a: &mut Vec<RealDom>, b: Rc<Vec<VDom>>) -> Vec<RealDom> {
    /*
        teraz kwestia jak zsynchronizować te dzieci

        Component-y reuzywamy

        najpierw porządkujemy koleność
            przenoszenie
            tworzenie nowych
            kasowanie nieaktualnych
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



fn renderToNode(anchor: DomAnchor, realDom: BoxRefCell<Vec<RealDom>>, computed: Computed<Rc<Vec<VDom>>>) -> Client {
    let subscription: Client = computed.subscribe(move |appVDom| {
        let anchor = anchor.clone();

        realDom.change(
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

//lib
fn startApp<T: 'static>(deps: Dependencies, param: T, render: fn(&T) -> Vec<VDom>) -> Client {
    let anchor = DomAnchor::root();

    let render /* (Fn() -> Rc<Vec<VDom>> */ = move || Rc::new(render(&param));
    let vDomComputed: Computed<Rc<Vec<VDom>>> = deps.from(render);

    let realDom: BoxRefCell<Vec<RealDom>> = BoxRefCell::new(Vec::new());

    //let vDomComputed: Computed<Vec<VDom>> = deps.from(move || render(&param));\

    let subscription = renderToNode(anchor, realDom, vDomComputed);
    subscription
}

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


fn app() -> Client {
    let root: Dependencies = Dependencies::new();
    let appState = AppState::new(&root);

    let subskrybcjaApp = startApp(root, appState, glownaFunkcjaRenderujaca);
    subskrybcjaApp
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
