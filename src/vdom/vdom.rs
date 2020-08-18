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
    //TODO
}

struct ComponentId {
    idComputed: GraphId,        //id tego computed
    idFunction: u64,            //id tej konkretnej funkcji statycznej (renderującej komponent)
}

struct Component {
    id: ComponentId,
    render: Box<dyn Fn() -> Vec<VDom>>
}

fn newComponent<T: Debug>(
    root: Dependencies,
    params: Computed<T>,
    render: fn(T) -> Vec<VDom>
) -> Component {
    let clientId = 4;   //TODO
    //let getValue = root.wrapGetValue(render, clientId);
    // to trzeba zamienic na subksrybcje
    // trzeba wystawic jakas funkcje subskryubujaca na funkcje (autorun)
    todo!();
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
        id: ComponentId,
        render: fn() -> Vec<VDom>
    },
    TestA {
        name: String
    }
}

fn aa(a: VDomNode) {

}

struct RealDomNode {
    name: String,
    attr: HashMap<String, String>,
    child: Vec<RealDom>,
    idDom: u64,                             //id realnego doma
                                            // --> getAllChild --> zwraca tablice ktora zawiera tylko ten elem vec!(idDom)
}

enum RealDom {
    Node {
        node: RealDomNode,
    },
    Text {
        value: String,
        idDom: u64,                             //id realnego doma
                                                // --> getAllChild --> zwraca tablice ktora zawiera tylko ten elem vec!(idDom)
    },
    Component {
        id: ComponentId,
        subscription: Client,                   //Subskrybcją, kryje się funkcja, która odpalana (na zmianę), wstawia coś do pojemnika child
        child: BoxRefCell<Vec<RealDom>>,
        idParent: RealNodeId,                   //parent
        idPrev: Option<RealNodeId>              // --> getAllChild --> Vec<u64>     przenosząc element przenosimy całą kolekcję elementów
    }
}

// fn applyNewView(dom_a: &RealDom, dom_b: &VDom) {
//     //...
//     /*
//         zeby przystąpić do synchronizaczji dwóch elementów, typ węzła musi się zgadzać
//             RealDom::name musi mieć takie samo jak VDom:name
//             typ RealDom:Text i typ VDom:Text

//             Component, moemy albo reuzyc, albo nie.
//             Ten element bedzie swietny ze wzgledu na keszowanie jego zawartosci
        

//         najpierw porządkujemy koleność
//             przenoszenie
//             tworzenie nowych
//             kasowanie nieaktualnych

//         potem następuej rekurencyjne wywołanie funkcji applyNewViewNode
//     */
// }


/*
    Główny root aplikacji, powinien być niezmiennym i niemodyfikowalnym węzłem
    Od niego zaczynamy zawsze 
*/

fn applyNewViewNode(idParent: RealNodeId, a: Vec<RealDom>, b: Vec<VDom>) {
    
    /*
        synchronizujemy atrybuty

        potem trzeba będzie zsynchronizować eventy podpięte pod ten węzeł

        potem przechodzimy do synchronizowania dzieci
        przenosząc prawdziwy element, od razu trzeba wysłać mu informację o tym jaki jest teraz obecnie jego poprzedzający element
    */
}

/*
    dzieci z A
    dzieci z B

    teraz kwestia jak zsynchronizować te dzieci

    dzieci z A, które 
*/


/*
    Vec<RealDom> --- lista, która powinna pozwalać na wstawienie elementu w jakimś określonym miejscu

    mona się pokusić o minigrf ...
    po to zeby dało się wstawiać elementy w odpowiedniej kolejności


    Przenosząc elemenent w kolekcji, trzeba 
*/



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
