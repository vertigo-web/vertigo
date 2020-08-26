use std::rc::Rc;
use virtualdom::computed::BoxRefCell::BoxRefCell;
use crate::vdom::{
    DomDriver::{
        DomDriver::DomDriver,
    },
    models::{
        RealDom::RealDom,
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

struct RealDomChildInner {
    domDriver: DomDriver,
    state: RealDomChildState,
}

impl RealDomChildInner {
    pub fn new(domDriver: DomDriver, comment: RealDomComment) -> RealDomChildInner {
        RealDomChildInner {
            domDriver,
            state: RealDomChildState::Empty {
                comment
            }
        }
    }

    pub fn newWithParent(driver: DomDriver, parent: RealDomId) -> RealDomChildInner {
        let nodeComment = RealDomComment::new(driver.clone(), "".into());
        driver.addChild(parent, nodeComment.idDom.clone());
        let nodeList = RealDomChildInner::new(driver,  nodeComment);

        nodeList
    }

    pub fn newDetached(driver: DomDriver) -> RealDomChildInner {
        let nodeComment = RealDomComment::new(driver.clone(), "".into());
        let nodeList = RealDomChildInner::new(driver,  nodeComment);

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
}

pub struct RealDomChild {
    inner: Rc<BoxRefCell<RealDomChildInner>>,
}

impl RealDomChild {
    pub fn newWithParent(driver: DomDriver, parent: RealDomId) -> RealDomChild {
        RealDomChild {
            inner: Rc::new(BoxRefCell::new(
                RealDomChildInner::newWithParent(driver, parent)
            ))
        }
    }

    pub fn newDetached(driver: DomDriver) -> RealDomChild {
        RealDomChild {
            inner: Rc::new(BoxRefCell::new(
                RealDomChildInner::newDetached(driver)
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
}

impl Clone for RealDomChild {
    fn clone(&self) -> Self {
        RealDomChild {
            inner: self.inner.clone()
        }
    }
}