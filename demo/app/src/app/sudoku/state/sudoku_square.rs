use super::tree_box::{ThreeBox, TreeBoxIndex};

pub struct SudokuSquare<T: Clone> {
    data: ThreeBox<ThreeBox<T>>,
}

impl<T: Clone> SudokuSquare<T> {
    pub fn new(data: ThreeBox<ThreeBox<T>>) -> SudokuSquare<T> {
        SudokuSquare { data }
    }

    pub fn create_with_iterator<F: Fn(TreeBoxIndex, TreeBoxIndex) -> T>(create: F) -> SudokuSquare<T> {
        SudokuSquare::new(ThreeBox::create_with_iterator(|level0x| {
            ThreeBox::create_with_iterator(|level0y| create(level0x, level0y))
        }))
    }

    pub fn get_from(&self, x: TreeBoxIndex, y: TreeBoxIndex) -> &T {
        self.data.get_from(x).get_from(y)
    }
}

impl<T: Clone> Clone for SudokuSquare<T> {
    fn clone(&self) -> Self {
        SudokuSquare {
            data: self.data.clone(),
        }
    }
}
