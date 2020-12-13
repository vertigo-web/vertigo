use std::rc::Rc;
use super::tree_box::{ThreeBox, TreeBoxIndex};

pub struct SudokuSquare<T> {
    data: Rc<ThreeBox<ThreeBox<T>>>,
}

impl<T> SudokuSquare<T> {
    pub fn new(data: ThreeBox<ThreeBox<T>>) -> SudokuSquare<T> {
        SudokuSquare {
            data: Rc::new(data),
        }
    }

    pub fn createWithIterator<F: Fn(TreeBoxIndex, TreeBoxIndex) -> T>(create: F) -> SudokuSquare<T> {
        SudokuSquare::new(
            ThreeBox::createWithIterator(|level0x| {
                ThreeBox::createWithIterator(|level0y| {
                    create(level0x, level0y)
                })
            })
        )
    }

    pub fn getFrom(&self, x: TreeBoxIndex, y: TreeBoxIndex) -> Rc<T> {
        self.data.getFrom(x).getFrom(y)
    }
}

impl<T> Clone for SudokuSquare<T> {
    fn clone(&self) -> Self {
        SudokuSquare {
            data: self.data.clone(),
        }
    }
}

// type SudokuSquareCallbackType<T> = (x: TreeBoxIndexType, y: TreeBoxIndexType, value: T) => void;

// type ToupleType = [TreeBoxIndexType, TreeBoxIndexType];

// const IteratorIndex: Array<ToupleType> = [
//     [0, 0],
//     [1, 0],
//     [2, 0],
//     [0, 1],
//     [1, 1],
//     [2, 1],
//     [0, 2],
//     [1, 2],
//     [2, 2],
// ];

// forEach(callback: SudokuSquareCallbackType<T>) {
//     for (const [x,y] of IteratorIndex) {
//         callback(x, y, this.data.getFrom(x).getFrom(y));
//     }
// }
