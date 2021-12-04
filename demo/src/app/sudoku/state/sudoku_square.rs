use std::cmp::PartialEq;

use super::tree_box::{ThreeBox, TreeBoxIndex};

#[derive(PartialEq)]
pub struct SudokuSquare<T: PartialEq + Clone> {
    data: ThreeBox<ThreeBox<T>>,
}

impl<T: PartialEq + Clone> SudokuSquare<T> {
    pub fn new(data: ThreeBox<ThreeBox<T>>) -> SudokuSquare<T> {
        SudokuSquare {
            data,
        }
    }

    pub fn create_with_iterator<F: Fn(TreeBoxIndex, TreeBoxIndex) -> T>(create: F) -> SudokuSquare<T> {
        SudokuSquare::new(
            ThreeBox::create_with_iterator(|level0x| {
                ThreeBox::create_with_iterator(|level0y| {
                    create(level0x, level0y)
                })
            })
        )
    }

    pub fn get_from(&self, x: TreeBoxIndex, y: TreeBoxIndex) -> T {
        self.data.get_from(x).get_from(y)
    }
}

impl<T: PartialEq + Clone> Clone for SudokuSquare<T> {
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
