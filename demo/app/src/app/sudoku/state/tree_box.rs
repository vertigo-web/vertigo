#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TreeBoxIndex {
    First,
    Middle,
    Last,
}

impl TreeBoxIndex {
    pub fn variants() -> Vec<TreeBoxIndex> {
        vec![TreeBoxIndex::First, TreeBoxIndex::Middle, TreeBoxIndex::Last]
    }
}

#[derive(PartialEq, Clone)]
pub struct ThreeBox<T: PartialEq + Clone> {
    data0: T,
    data1: T,
    data2: T,
}

impl<T: PartialEq + Clone> ThreeBox<T> {
    pub fn new(data0: T, data1: T, data2: T) -> ThreeBox<T> {
        ThreeBox { data0, data1, data2 }
    }

    pub fn create_with_iterator<F: Fn(TreeBoxIndex) -> T>(create: F) -> ThreeBox<T> {
        ThreeBox::new(
            create(TreeBoxIndex::First),
            create(TreeBoxIndex::Middle),
            create(TreeBoxIndex::Last),
        )
    }

    pub fn get_from(&self, index: TreeBoxIndex) -> T {
        match index {
            TreeBoxIndex::First => self.data0.clone(),
            TreeBoxIndex::Middle => self.data1.clone(),
            TreeBoxIndex::Last => self.data2.clone(),
        }
    }
}
