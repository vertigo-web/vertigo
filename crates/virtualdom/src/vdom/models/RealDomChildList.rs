use std::rc::Rc;
use crate::computed::BoxRefCell::BoxRefCell;
use crate::vdom::{
    DomDriver::{
        DomDriver::DomDriver,
    },
    models::{
        RealDom::RealDom,
        RealDomNode::RealDomNode,
        RealDomText::RealDomText,
        RealDomId::RealDomId,
        RealDomComment::RealDomComment,
    },
};

enum RealDomChildState {
    Empty {
        comment: RealDomComment,
    },
    List {
        first: RealDom,
        child: Vec<RealDom>,
    }
}

impl RealDomChildState {
    fn isEmpty(&self) -> bool {
        match self {
            RealDomChildState::Empty { .. } => true,
            RealDomChildState::List { .. } => false,
        }
    }
}

struct RealDomChildListInner {
    domDriver: DomDriver,
    state: RealDomChildState,
}

impl RealDomChildListInner {
    pub fn new(domDriver: DomDriver, comment: RealDomComment) -> RealDomChildListInner {
        RealDomChildListInner {
            domDriver,
            state: RealDomChildState::Empty {
                comment
            }
        }
    }

    pub fn newWithParent(driver: DomDriver, parent: RealDomId) -> RealDomChildListInner {
        let nodeComment = RealDomComment::new(driver.clone(), "".into());
        driver.addChild(parent, nodeComment.idDom.clone());
        let nodeList = RealDomChildListInner::new(driver,  nodeComment);

        nodeList
    }

    pub fn newAfter(driver: DomDriver, afterRef: RealDomId) -> RealDomChildListInner {
        let nodeComment = RealDomComment::new(driver.clone(), "".into());
        driver.insertAfter(afterRef, nodeComment.idDom.clone());
        let nodeList = RealDomChildListInner::new(driver,  nodeComment);

        nodeList
    }

    pub fn extract(&mut self) -> Vec<RealDom> {
        let isEmpty = self.state.isEmpty();

        if isEmpty {
            return Vec::new();
        }
    
        let firstChildId = self.firstChildId();
        let nodeComment = RealDomComment::new(self.domDriver.clone(), "".into());
        self.domDriver.insertBefore(firstChildId, nodeComment.idDom.clone());

        let prevState = std::mem::replace(&mut self.state, RealDomChildState::Empty {
            comment: nodeComment
        });

        match prevState {
            RealDomChildState::Empty { .. } => {
                unreachable!();
            },
            RealDomChildState::List { first, child } => {
                let mut out = Vec::new();
                out.push(first);
                out.extend(child);
                out
            }
        }
    }

    pub fn append(&mut self, newChild: RealDom) {
        let childIds = newChild.childIds();
        let mut refId = self.lastChildId();

        for item in childIds {
            self.domDriver.insertAfter(refId, item.clone());
            refId = item;
        }

        let isEmpty = self.state.isEmpty();

        if isEmpty {
            self.state = RealDomChildState::List {
                first: newChild,
                child: Vec::new(),
            };
        } else {
            match &mut self.state {
                RealDomChildState::Empty { .. } => {
                    unreachable!();
                },
                RealDomChildState::List { child, .. } => {
                    (*child).push(newChild);
                }
            };
        }
    }

    pub fn firstChildId(&self) -> RealDomId {
        match &self.state {
            RealDomChildState::Empty { comment } => {
                comment.idDom.clone()
            },
            RealDomChildState::List { first, .. } => {
                first.firstChildId()
            }
        }
    }

    pub fn lastChildId(&self) -> RealDomId {
        match &self.state {
            RealDomChildState::Empty { comment } => {
                comment.idDom.clone()
            },
            RealDomChildState::List { first, child, .. } => {
                let last = child.last();

                if let Some(last) = last {
                    last.lastChildId()
                } else {
                    first.lastChildId()
                }
            }
        }
    }

    pub fn childIds(&self) -> Vec<RealDomId> {
        let mut out = Vec::new();

        match &self.state {
            RealDomChildState::Empty { comment } => {
                out.push(comment.idDom.clone())
            },
            RealDomChildState::List { first, child } => {
                out.extend(first.childIds());

                for childItem in child {
                    out.extend(childItem.childIds());
                }
            }
        }

        out
    }

    fn createNode(&self, name: &'static str) -> RealDomNode {
        RealDomNode::new(self.domDriver.clone(), name, self.lastChildId())
    }

    fn createText(&self, name: String) -> RealDomText {
        let node = RealDomText::new(self.domDriver.clone(), name);
        self.domDriver.insertAfter(self.lastChildId(), node.idDom.clone());
        node
    }

    fn createChildList(&self) -> RealDomChildListInner {
        RealDomChildListInner::newAfter(self.domDriver.clone(), self.lastChildId())
    }
}

pub struct RealDomChildList {
    inner: Rc<BoxRefCell<RealDomChildListInner>>,
}

impl RealDomChildList {
    pub fn newWithParent(driver: DomDriver, parent: RealDomId) -> RealDomChildList {
        RealDomChildList {
            inner: Rc::new(BoxRefCell::new(
                RealDomChildListInner::newWithParent(driver, parent)
            ))
        }
    }

    pub fn extract(&self) -> Vec<RealDom> {
        self.inner.changeNoParams(|state| {
            state.extract()
        })
    }

    pub fn append(&self, child: RealDom) {
        self.inner.change(child, |state, child| {
            state.append(child)
        })
    }

    pub fn firstChildId(&self) -> RealDomId {
        self.inner.get(|state| {
            state.firstChildId()
        })
    }

    pub fn lastChildId(&self) -> RealDomId {
        self.inner.get(|state| {
            state.lastChildId()
        })
    }

    pub fn childIds(&self) -> Vec<RealDomId> {
        self.inner.get(|state| {
            state.childIds()
        })
    }

    pub fn getDomDriver(&self) -> DomDriver {
        self.inner.get(|state| {
            state.domDriver.clone()
        })
    }

    pub fn createNode(&self, name: &'static str) -> RealDomNode {
        self.inner.getWithContext(name, |state, name| {
            state.createNode(name)
        })
    }

    pub fn createText(&self, name: String) -> RealDomText {
        self.inner.getWithContext(name, |state, name| {
            state.createText(name)
        })
    }

    pub fn createChildList(&self) -> RealDomChildList {
        let newChildList = self.inner.get(|state| {
            state.createChildList()
        });

        RealDomChildList {
            inner: Rc::new(BoxRefCell::new(newChildList))
        }
    }
}

impl Clone for RealDomChildList {
    fn clone(&self) -> Self {
        RealDomChildList {
            inner: self.inner.clone()
        }
    }
}