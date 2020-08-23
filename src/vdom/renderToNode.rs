
use std::rc::Rc;
use std::collections::{
    HashMap,
    VecDeque,
};

use crate::{
    lib::{
        Client::Client,
        Computed::Computed,
    },
    vdom::{
        models::{
            VDom::{VDom, VDomNode},
            RealDomChild::RealDomChild,
            RealDomNode::RealDomNode,
            RealDomText::RealDomText,
            RealDomComponent::RealDomComponent,
            VDomComponentId::VDomComponentId,
        }
    }
};

/*
    Główny root aplikacji, powinien być niezmiennym i niemodyfikowalnym węzłem
    Od niego zaczynamy zawsze (numer 1)
*/

fn applyNewViewChild(target: RealDomChild, newVersion: Vec<VDom>) -> VecDeque<(RealDomChild, Vec<VDom>)> {

    let realNode: HashMap<String, Vec<RealDomNode>> = HashMap::new();
    let realText: HashMap<String, Vec<RealDomText>> = HashMap::new();
    let realComponent: HashMap<VDomComponentId, Vec<RealDomComponent>> = HashMap::new();

    let realChil = target.extract();

    for item in realChil {
        //trzeba przerzucić realny dom item do któregoś z powyszych kubełków
    }


    let mut out = VecDeque::new();

    for item in newVersion.iter() {

        match item {
            VDom::Node { node } => {
                let child = node.child.clone();

            },
            VDom::Text { value } => {

            },
            VDom::Component { node } => {

            }
        }
    }

    todo!();

    out

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
    //todo!();
}


fn applyNewViewNode(om_a: &RealDomNode, dom_b: &VDomNode) {
    /*
        zeby przystąpić do synchronizaczji dwóch elementów, typ węzła musi się zgadzać
            RealDom::name musi mieć takie samo jak VDom:name
        
        synchronizujemy atrybuty

        potem trzeba będzie zsynchronizować eventy podpięte pod ten węzeł

        potem przechodzimy do synchronizowania dzieci
    */
    todo!();
}





pub fn renderToNode(target: RealDomChild, computed: Computed<Rc<Vec<VDom>>>) -> Client { 
    let subscription: Client = computed.subscribe(move |newVersion| {
        let mut syncTodo: VecDeque<(RealDomChild, Vec<VDom>)> = applyNewViewChild(
            target.clone(), 
            (*(*newVersion)).clone()
        );

        loop {
            let first = syncTodo.pop_front();

            match first {
                Some((realList, virtualList)) => {
                    let newListTodo = applyNewViewChild(realList, virtualList);
                    syncTodo.extend(newListTodo);
                },
                None => {
                    return;
                }
            }
        }
    });

    subscription
}

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
