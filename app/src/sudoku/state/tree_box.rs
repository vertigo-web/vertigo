use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub enum TreeBoxIndex {
    First,
    Middle,
    Last,
}

impl TreeBoxIndex {
    pub fn variants() -> Vec<TreeBoxIndex> {
        vec!(TreeBoxIndex::First, TreeBoxIndex::Middle, TreeBoxIndex::Last)
    }
}

pub struct ThreeBox<T> {
    data0: Rc<T>,
    data1: Rc<T>,
    data2: Rc<T>
}

impl<T> ThreeBox<T> {
    pub fn new(data0: T, data1: T, data2: T) -> ThreeBox<T> {
        ThreeBox {
            data0: Rc::new(data0),
            data1: Rc::new(data1),
            data2: Rc::new(data2),
        }
    }

    pub fn createWithIterator<F: Fn(TreeBoxIndex) -> T>(create: F) -> ThreeBox<T> {
        ThreeBox::new(
            create(TreeBoxIndex::First),
            create(TreeBoxIndex::Middle),
            create(TreeBoxIndex::Last)
        )
    }

    pub fn getFrom(&self, index: TreeBoxIndex) -> Rc<T> {
        match index {
            TreeBoxIndex::First => self.data0.clone(),
            TreeBoxIndex::Middle => self.data1.clone(),
            TreeBoxIndex::Last => self.data2.clone(),
        }
    }
}
