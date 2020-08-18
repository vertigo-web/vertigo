use std::fmt::Debug;
use std::collections::HashMap;
use crate::lib::{
    Client::Client,
    GraphId::GraphId,
    Computed::Computed,
    Dependencies::Dependencies,
    BoxRefCell::BoxRefCell,
};

struct RealNodeId {
    id: u64,
}

struct ComponentId {
    idComputed: GraphId,        //id tego computed
    idFunction: u64,            //id tej konkretnej funkcji statycznej (renderującej komponent)
}

struct Component {
    id: ComponentId,
    render: Box<dyn Fn() -> Vec<VDom>>
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
        child: Rc<{                             //Ten element będzie przekazany do funkcji renderującej ---> a potem subskrybcja będzie zapisana do zmiennej subscription
            child: BoxRefCell<Vec<RealDom>>,
            idParent: RealNodeId,               //prawdopodobnie będzie konieczne. Ale ten id moze byc utworzony przy stworzeniu noda. Nie będzie zmieniany.
        }>
    }
}

enum DomAnchor {
    Parent(RealNodeId),             //oznacza ze zaczynamy wstawiac elementy jako pierwsze dziecko
    RefPrev(RealNodeId),            //pokazuje poprzedni element przed zakresem
}

impl DomAnchor {
    fn root() -> DomAnchor {
        DomAnchor::Parent(1)
    }
}

/*
RealDom::Node - DomAnchor::Parent(), będzie odnosnikiem
RealDom::Component - DomAnchor::RefPrev()
*/


fn newComponent<T: Debug>(root: Dependencies, params: Computed<T>, render: fn(T) -> Vec<VDom>) -> Component {
    let clientId = 4;   //TODO
    //let getValue = root.wrapGetValue(render, clientId);
    // to trzeba zamienic na subksrybcje, trzeba wystawic jakas funkcje subskryubujaca na funkcje (autorun)
    todo!();
}

/*
    Główny root aplikacji, powinien być niezmiennym i niemodyfikowalnym węzłem
    Od niego zaczynamy zawsze (numer 1)
*/

fn applyNewViewChild(anchor: DomAnchor, a: Vec<RealDom>, b: Vec<VDom>) {
    /*
        teraz kwestia jak zsynchronizować te dzieci

        Component-y reuzywamy

        najpierw porządkujemy koleność
            przenoszenie
            tworzenie nowych
            kasowanie nieaktualnych
    */
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


fn startApp<T>(deps: Dependencies,, param: T, render: fn(&T) -> Vec<VDom>) -> Client {
    let mut prevAppVDom: Vec<VDom> = Vec::new();

    let appVdomCom: Computed<Vec<VDom>> = Computed::new(deps, (move || {
        render(&param)
    });

    let subscription: Client = appVdomCom.subscribe(move |appVDom| {
        renderApp(
            DomAnchor::root(),
            prevAppVDom,
            appVDom
        );
    
        prevAppVDom = appVDom;    
    })

    subscription
}

let root: Dependencies = Dependencies::new();

let appState = AppState::new(&root);

let subskrybcjaApp = startApp(root, appState, glownaFunkcjaRenderujaca);

//po wystartowaniu subskrybcjaApp tą zmienną trzeba wpakować w zmienną globalną zeby nie stracić subskrybcji

fn glownaFunkcjaRenderujaca(appState: AppState) -> Vec<VDom> {
    todo!();
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
